use {
    crate::BoxResult,
    async_trait::async_trait,
    std::{
        collections::{HashMap, HashSet},
        io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult},
    },
    tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
};

/// Additional methods for Lucene decoding on top of the standard `AsyncRead` trait.
///
/// # Lucene variable length integer encoding
///
/// Lucene often uses a variable-length integer encoding scheme called VByte. VByte is a format intended for positive
/// where the high-order bit of each byte indicates whether more bytes remain to be read. The low-order seven bits
/// are appended as increasingly more significant bits in the resulting integer value. Thus values from 0 to 127 are
/// stored in a single byte, values from 128 to 16,383 are stored in two bytes, and so on.
///
/// Negative numbers are supported but always use the maximum number of bytes for the size (either five or nine).
///
/// ## VByte encoding example for i32
///
/// ```text
/// ╔════════════════╤══════════╤══════════╤══════════╤══════════╤══════════╗
/// ║      Value     │  Byte 1  │  Byte 2  │  Byte 3  │  Byte 4  │  Byte 5  ║
/// ╟────────────────┼──────────┼──────────┼──────────┼──────────┼──────────╢
/// ║              0 │ 00000000 │          │          │          │          ║
/// ║              1 │ 00000001 │          │          │          │          ║
/// ║              2 │ 00000010 │          │          │          │          ║
/// ║       ...      │          │          │          │          │          ║
/// ║            127 │ 01111111 │          │          │          │          ║
/// ║            128 │ 10000000 │ 00000001 │          │          │          ║
/// ║            129 │ 10000001 │ 00000001 │          │          │          ║
/// ║            130 │ 10000010 │ 00000001 │          │          │          ║
/// ║       ...      │          │          │          │          │          ║
/// ║         16_383 │ 11111111 │ 01111111 │          │          │          ║
/// ║         16_384 │ 10000000 │ 10000000 │ 00000001 │          │          ║
/// ║         16_385 │ 10000001 │ 10000000 │ 00000001 │          │          ║
/// ║       ...      │          │          │          │          │          ║
/// ║    268_435_455 | 11111111 │ 11111111 │ 11111111 │ 01111111 │          ║
/// ║    268_435_456 | 10000000 │ 10000000 │ 10000000 │ 10000000 │ 00000001 ║
/// ║       ...      │          │          │          │          │          ║
/// ║  2_147_483_647 │ 11111111 │ 11111111 │ 11111111 │ 11111111 │ 00000111 ║
/// ║ -2_147_483_648 │ 10000000 │ 10000000 │ 10000000 │ 10000000 │ 00001000 ║
/// ║       ...      │          │          │          │          │          ║
/// ║             -1 │ 11111111 │ 11111111 │ 11111111 │ 11111111 │ 00001111 ║
/// ╚════════════════╧══════════╧══════════╧══════════╧══════════╧══════════╝
/// ```
///
/// ## Rust notes
///
/// Although it is theoretically possible to support unsigned encodings here, _we intentionally do not to maintain Java
/// compatibility_. A `u31` type (e.g. from the [ux crate](https://docs.rs/ux/latest/ux/)), for example, could be
/// supported on the write side, but the read side presents problems since it is possible for a high bit to be set
/// (representing a negative-valued `i32`) even if it makes no sense for the value to be negative. This checking is,
/// alas, forced onto the (internal) consumer of this API.

#[async_trait(?Send)]
pub trait EncodingReadExt: AsyncReadExt + Unpin {
    /// Reads a short string (0-255 bytes).
    ///
    /// Reads a string as UTF-8 encoded bytes. One byte is read for the length. Then that number of UTF-8 bytes is read.
    ///
    /// # Errors
    /// This method will return an error if the string is not a valid UTF-8 string or an underlying I/O error occurs.
    async fn read_short_string(&mut self) -> BoxResult<String> {
        let str_len = self.read_u8().await? as usize;
        let mut byte_buf = vec![0u8; str_len];
        self.read_exact(&mut byte_buf).await?;
        let s = String::from_utf8(byte_buf)?;
        Ok(s)
    }

    /// Reads a string.
    ///
    /// Reads a string as UTF-8 encoded bytes. Between one and five bytes is read for the length. Then that many number
    /// of UTF-8 bytes is read.
    ///
    /// # Errors
    /// This method will return an error if the length is negative, the string is not a valid UTF-8 string, or an
    /// underlying I/O error occurs.
    async fn read_string(&mut self) -> BoxResult<String> {
        let str_len = self.read_vi32().await?;
        let str_len = str_len.try_into()?;
        let mut bytes = vec![0u8; str_len];
        self.read_exact(&mut bytes).await?;
        let s = String::from_utf8(bytes)?;
        Ok(s)
    }

    /// Reads an i32 stored in variable-length VByte format. Reads between one and five bytes. Smaller values
    /// take fewer bytes. See the documentation for [EncodingReadExt] for details on the VByte format.
    ///
    /// # Errors
    /// This method will return an error if the value cannot fit into an i32 or an underlying I/O error occurs.
    async fn read_vi32(&mut self) -> IoResult<i32> {
        let mut b = self.read_u8().await?;
        let mut result = (b & 0x7F) as i32;
        let mut shift = 7;
        let mut n_read = 1;

        while (b & 0x80) != 0 {
            if n_read >= 5 {
                return Err(IoError::new(IoErrorKind::InvalidData, "Cannot read a vi32 larger than 5 bytes"));
            }

            b = self.read_u8().await?;
            n_read += 1;
            result |= ((b & 0x7F) as i32) << shift;
            shift += 7;
        }

        Ok(result)
    }

    /// Reads a v64 stored in variable-length VByte format. Reads between one and nine bytes. Smaller values
    /// take fewer bytes. See the documentation for [EncodingReadExt] for details on the VByte format.
    ///
    /// # Errors
    /// This method will return an error if the value cannot fit into an i64 or an underlying I/O error occurs.
    async fn read_vi64(&mut self) -> IoResult<i64> {
        let mut b = self.read_u8().await?;
        let mut result = (b & 0x7F) as i64;
        let mut shift = 7;
        let mut n_read = 1;

        while (b & 0x80) != 0 {
            if n_read >= 9 {
                return Err(IoError::new(IoErrorKind::InvalidData, "Cannot read a vi64 larger than 9 bytes"));
            }

            b = self.read_u8().await?;
            n_read += 1;
            result |= ((b & 0x7F) as i64) << shift;
            shift += 7;
        }

        Ok(result)
    }

    /// Reads a map of strings to strings.
    ///
    /// First, the number of entries is read using [EncodingReadExt::read_vi32]. Then that many entries are read.
    /// Each entry consists of a string key followed by a string value, each read using [EncodingReadExt::read_string].
    ///
    /// If the number of entries is negative it is treated as zero. This matches the behavior of the Java
    /// implementation.
    ///
    /// # Errors
    /// This method will return an error if the number of any [EncodingReadExt::read_vi32] or [EncodingReadExt::read_string]
    /// call fails.
    async fn read_string_map(&mut self) -> BoxResult<HashMap<String, String>> {
        let num_entries = self.read_vi32().await?;
        let num_entries = if num_entries < 0 {
            0
        } else {
            num_entries as usize
        };

        let mut map = HashMap::with_capacity(num_entries);
        for _ in 0..num_entries {
            let key = self.read_string().await?;
            let value = self.read_string().await?;
            map.insert(key, value);
        }

        Ok(map)
    }

    /// Reads a set of strings.
    ///
    /// First, the number of entries is read using [EncodingReadExt::read_vi32]. Then that many strings are read
    /// using [EncodingReadExt::read_string].
    ///
    /// If the number of entries is negative it is treated as zero. This matches the behavior of the Java
    /// implementation.
    ///
    /// # Errors
    /// This method will return an error if the number of any [EncodingReadExt::read_vi32] or [EncodingReadExt::read_string]
    /// call fails.
    async fn read_string_set(&mut self) -> BoxResult<HashSet<String>> {
        let num_entries = self.read_vi32().await?;
        let num_entries = if num_entries < 0 {
            0
        } else {
            num_entries as usize
        };

        let mut set = HashSet::with_capacity(num_entries);
        for _ in 0..num_entries {
            let key = self.read_string().await?;
            set.insert(key);
        }

        Ok(set)
    }
}

impl<R: AsyncRead + Unpin + ?Sized> EncodingReadExt for R {}

/// Additional methods for Lucene encoding on top of the standard `Write` trait.
///
/// See [EncodingReadExt] for a decription of the variable length integer encoding used by Lucene.
#[async_trait(?Send)]
pub trait EncodingWriteExt: AsyncWriteExt + Unpin {
    /// Writes a short string (0-255 bytes).
    ///
    /// Writes a string as UTF-8 encoded bytes. One byte is written for the length in bytes. Then that number of UTF-8
    /// bytes is written.
    ///
    /// # Errors
    /// This method will return an error if the string is not less than 256 bytes or an underlying I/O error occurs.
    async fn write_short_string(&mut self, s: &str) -> IoResult<()> {
        let len = s.len();
        if len > u8::MAX as usize {
            return Err(IoError::new(IoErrorKind::InvalidData, "String too long"));
        }
        self.write_u8(len as u8).await?;
        self.write_all(s.as_bytes()).await?;
        Ok(())
    }

    /// Writes a string.
    ///
    /// Writes strings as UTF-8 encoded bytes. First the length, in bytes, is written as a variable length integer
    /// ([EncodingWriteExt::write_vi32]), followed by the bytes.
    async fn write_string(&mut self, s: &str) -> IoResult<()> {
        let len = s.len();
        if len > i32::MAX as usize {
            return Err(IoError::new(IoErrorKind::InvalidData, "String too long"));
        }
        self.write_vi32(len as i32).await?;
        self.write_all(s.as_bytes()).await?;
        Ok(())
    }

    /// Writes an i32 in a variable-length format. Writes between one and three bytes. Smaller values
    /// take fewer bytes.
    ///
    /// See [EncodingReadExt] for a decription of the variable length integer encoding used by Lucene.
    ///
    /// # Errors
    /// This method will return an error if an underlying I/O error occurs.
    async fn write_vi32(&mut self, i: i32) -> IoResult<()> {
        let mut i = i as u32;
        while (i & !0x7f) != 0 {
            self.write_u8((i as u8 & 0x7f) | 0x80).await?;
            i >>= 7;
        }

        self.write_u8(i as u8).await?;
        Ok(())
    }

    /// Writes an i64 in a variable-length format. Writes between one and nine bytes. Smaller values
    /// take fewer bytes. Negative numbers are supported but should be avoided.
    ///
    /// See [EncodingReadExt] for a decription of the variable length integer encoding used by Lucene.
    ///
    /// # Errors
    /// This method will return an error if an underlying I/O error occurs.
    async fn write_vi64(&mut self, i: i64) -> IoResult<()> {
        let mut i = i as u64;

        while (i & !0x7f) != 0 {
            self.write_u8((i as u8 & 0x7f) | 0x80).await?;
            i >>= 7;
        }

        self.write_u8(i as u8).await?;
        Ok(())
    }

    /// Writes a hash map of strings to strings.
    ///
    /// First, the number of entries is written using [EncodingWriteExt::write_vi32]. Then that many entries are written.
    /// Each entry consists of a string key followed by a string value, each written using
    /// [EncodingWriteExt::write_string].
    ///
    /// # Errors
    /// This method will return an error if the number of any [EncodingWriteExt::write_vi32] or
    /// [EncodingWriteExt::write_string] call fails.
    async fn write_string_map(&mut self, map: &HashMap<String, String>) -> IoResult<()> {
        self.write_vi32(map.len() as i32).await?;
        for (key, value) in map {
            self.write_string(key).await?;
            self.write_string(value).await?;
        }
        Ok(())
    }

    /// Writes a set of strings.
    ///
    /// First, the number of entries is written using [EncodingWriteExt::write_vi32]. Then that many strings are written
    /// using [EncodingWriteExt::write_string].
    ///
    /// # Errors
    /// This method will return an error if the number of any [EncodingWriteExt::write_vi32] or
    /// [EncodingWriteExt::write_string] call fails.
    async fn write_string_set(&mut self, set: &HashSet<String>) -> IoResult<()> {
        self.write_vi32(set.len() as i32).await?;
        for value in set {
            self.write_string(value).await?;
        }
        Ok(())
    }
}

impl<W: AsyncWrite + Unpin + ?Sized> EncodingWriteExt for W {}

#[cfg(test)]
mod tests {
    use {
        crate::{codec::CodecHeader, io::EncodingWriteExt},
        pretty_assertions::assert_eq,
    };

    #[test_log::test(tokio::test)]
    async fn test_write_header() {
        let header = CodecHeader::new("test", 1).unwrap();
        let mut buf = Vec::new();
        header.write(&mut buf).await.unwrap();
        assert_eq!(buf, vec![0x3f, 0xd7, 0x6c, 0x17, 0x4, 0x74, 0x65, 0x73, 0x74, 0x0, 0x0, 0x0, 0x1]);
    }

    #[test_log::test(tokio::test)]
    async fn test_write_vi32() {
        let mut buf = Vec::new();

        for i in 0..127 {
            buf.write_vi32(i).await.unwrap();
            assert_eq!(buf, vec![i as u8]);
            buf.clear();
        }

        buf.write_vi32(128).await.unwrap();
        assert_eq!(buf, vec![0b1000_0000, 0b0000_0001]);
        buf.clear();

        buf.write_vi32(129).await.unwrap();
        assert_eq!(buf, vec![0b1000_0001, 0b0000_0001]);
        buf.clear();

        buf.write_vi32(16383).await.unwrap();
        assert_eq!(buf, vec![0b1111_1111, 0b0111_1111]);
        buf.clear();

        buf.write_vi32(16384).await.unwrap();
        assert_eq!(buf, vec![0b1000_0000, 0b1000_0000, 0b0000_0001]);
        buf.clear();

        buf.write_vi32(16385).await.unwrap();
        assert_eq!(buf, vec![0b1000_0001, 0b1000_0000, 0b0000_0001]);
        buf.clear();

        buf.write_vi32(2_147_483_647).await.unwrap();
        assert_eq!(buf, vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_0111]);
        buf.clear();

        buf.write_vi32(-2_147_483_648).await.unwrap();
        assert_eq!(buf, vec![0b1000_0000, 0b1000_0000, 0b1000_0000, 0b1000_0000, 0b0000_1000]);
        buf.clear();

        buf.write_vi32(-1).await.unwrap();
        assert_eq!(buf, vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_1111]);
    }

    #[test_log::test(tokio::test)]
    async fn test_write_string() {
        let mut buf = Vec::new();
        buf.write_string("hello").await.unwrap();
        assert_eq!(buf, vec![5, 104, 101, 108, 108, 111]);
    }
}
