use std::io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult, Write};

/// Constant to identify the start of a codec header.
pub const CODEC_MAGIC: [u8; 4] = [0x3f, 0xd7, 0x6c, 0x17];

/// Constant to identify the start of a codec footer -- bit inversion of [CODEC_MAGIC].
pub const FOOTER_MAGIC: [u8; 4] = [0xc0, 0x28, 0x93, 0xe8];

/// Writes a codec header, which records both a string to identify the file and a version number.
/// This header can be parsed and validated with [check_header].
///
/// CodecHeader --> Magic + CodecName + Version
///
/// * Magic (4 bytes): This identifies the start of the header and is always [CODEC_MAGIC].
/// * CodecName ([write_string]): This is a string to identify this file. This must be 127 bytes or less and in ASCII.
/// * Version (BE u32): Records the version of the file.

pub fn write_header<W: Write>(w: &mut W, codec: &str, version: u32) -> IoResult<usize> {
    if !codec.is_ascii() {
        return Err(IoError::new(IoErrorKind::InvalidData, "Codec name must be ASCII"));
    }

    if codec.len() >= 128 {
        return Err(IoError::new(IoErrorKind::InvalidData, "Codec name must be 127 bytes or less"));
    }

    let mut n_written = w.write(&CODEC_MAGIC)?;
    n_written += write_string(w, codec)?;
    n_written += w.write(&version.to_be_bytes())?;
    Ok(n_written)
}

/// Writes an i32 in a variable-length format. Writes between one and five bytes. Smaller values
/// take fewer bytes. Negative numbers are supported but should be avoided.
///
/// VByte is a variable-length format for positive integers is defined where the high-order bit
/// of each byte indicates whether more bytes remain to be read. The low-order seven bits are
/// appended as increasingly more significant bits in the resulting integer value. Thus values from
/// zero to 127 may be stored in a single byte, values from 128 to 16,383 may be stored in two
/// bytes, and so on.
///
/// VByte Encoding Example
///
/// ```ignore
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
/// This provides compression while still being efficient to decode.
pub fn write_vi32<W: Write>(w: &mut W, i: i32) -> IoResult<usize> {
    let mut i = i as u32;

    let mut n_written = 0;
    while (i & !0x7f) != 0 {
        n_written += w.write(&[(i as u8 & 0x7f) | 0x80])?;
        i >>= 7;
    }

    n_written += w.write(&[i as u8])?;
    Ok(n_written)
}

/// Writes a string.
///
/// Writes strings as UTF-8 encoded bytes. First the length, in bytes, is written as a variable length integer
/// ([write_vint]), followed by the bytes.
pub fn write_string<W: Write>(w: &mut W, s: &str) -> IoResult<usize> {
    let mut n_written = write_vi32(w, s.len() as i32)?;
    n_written += w.write(s.as_bytes())?;
    Ok(n_written)
}

#[cfg(test)]
mod tests {
    use {super::*, pretty_assertions::assert_eq, test_log::test};

    #[test]
    fn test_write_header() {
        let mut buf = Vec::new();
        write_header(&mut buf, "test", 1).unwrap();
        assert_eq!(buf, vec![0x3f, 0xd7, 0x6c, 0x17, 0x4, 0x74, 0x65, 0x73, 0x74, 0x0, 0x0, 0x0, 0x1]);
    }

    #[test]
    fn test_write_vi32() {
        let mut buf = Vec::new();

        for i in 0..127 {
            write_vi32(&mut buf, i).unwrap();
            assert_eq!(buf, vec![i as u8]);
            buf.clear();
        }

        write_vi32(&mut buf, 128).unwrap();
        assert_eq!(buf, vec![0b1000_0000, 0b0000_0001]);
        buf.clear();

        write_vi32(&mut buf, 129).unwrap();
        assert_eq!(buf, vec![0b1000_0001, 0b0000_0001]);
        buf.clear();

        write_vi32(&mut buf, 16383).unwrap();
        assert_eq!(buf, vec![0b1111_1111, 0b0111_1111]);
        buf.clear();

        write_vi32(&mut buf, 16384).unwrap();
        assert_eq!(buf, vec![0b1000_0000, 0b1000_0000, 0b0000_0001]);
        buf.clear();

        write_vi32(&mut buf, 16385).unwrap();
        assert_eq!(buf, vec![0b1000_0001, 0b1000_0000, 0b0000_0001]);
        buf.clear();

        write_vi32(&mut buf, 2_147_483_647).unwrap();
        assert_eq!(buf, vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_0111]);
        buf.clear();

        write_vi32(&mut buf, -2_147_483_648).unwrap();
        assert_eq!(buf, vec![0b1000_0000, 0b1000_0000, 0b1000_0000, 0b1000_0000, 0b0000_1000]);
        buf.clear();

        write_vi32(&mut buf, -1).unwrap();
        assert_eq!(buf, vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_1111]);
    }

    #[test]
    fn test_write_string() {
        let mut buf = Vec::new();
        write_string(&mut buf, "hello").unwrap();
        assert_eq!(buf, vec![5, 104, 101, 108, 108, 111]);
    }
}
