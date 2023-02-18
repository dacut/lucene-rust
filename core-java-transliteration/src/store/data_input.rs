use {
    crate::util::bit_util::ZigZag,
    std::{
        collections::{HashMap, HashSet},
        future::Future,
        io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult},
        mem::size_of,
        pin::Pin,
        slice::from_raw_parts_mut,
    },
};

/// Trait for performing read operations of Lucene's low-level data types.
///
/// This should be replaced entirely by [tokio::io::AsyncRead].
pub trait DataInput {
    /// Reads and returns a single byte.
    fn read_byte(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<u8>>>>;

    /// Reads a enough bytes to fill a slice exactly. If the end of the stream is reached before the
    /// slice is filled, an [std::io::ErrorKind::UnexpectedEof] error is returned.
    fn read_bytes(self: Pin<&mut Self>, buffer: &mut [u8]) -> Pin<Box<dyn Future<Output = IoResult<()>>>>;

    /// Reads two bytes and returns a u16 parsed from them in little-endian order.
    fn read_u16(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<u16>>>> {
        Box::pin(async move {
            let mut buffer = [0u8; 2];
            self.read_bytes(&mut buffer).await?;
            Ok(u16::from_le_bytes(buffer))
        })
    }

    /// Reads two bytes and returns an i16 parsed from them in little-endian order.
    fn read_i16(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<i16>>>> {
        Box::pin(async move {
            let mut buffer = [0u8; 2];
            self.read_bytes(&mut buffer).await?;
            Ok(i16::from_le_bytes(buffer))
        })
    }

    /// Reads four bytes and returns a u32 parsed from them in little-endian order.
    fn read_u32(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<u32>>>> {
        Box::pin(async move {
            let mut buffer = [0u8; 4];
            self.read_bytes(&mut buffer).await?;
            Ok(u32::from_le_bytes(buffer))
        })
    }

    /// Reads four bytes and returns an i32 parsed from them in little-endian order.
    fn read_i32(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<i32>>>> {
        Box::pin(async move {
            let mut buffer = [0u8; 4];
            self.read_bytes(&mut buffer).await?;
            Ok(i32::from_le_bytes(buffer))
        })
    }

    /// Reads eight bytes and returns a u64 parsed from them in little-endian order.
    fn read_u64(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<u64>>>> {
        Box::pin(async move {
            let mut buffer = [0u8; 8];
            self.read_bytes(&mut buffer).await?;
            Ok(u64::from_le_bytes(buffer))
        })
    }

    /// Reads eight bytes and returns an i64 parsed from them in little-endian order.
    fn read_i64(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<i64>>>> {
        Box::pin(async move {
            let mut buffer = [0u8; 8];
            self.read_bytes(&mut buffer).await?;
            Ok(i64::from_le_bytes(buffer))
        })
    }

    /// Reads enough i64s to fill the slice exactly. If the end of the stream is reached before the
    /// slice is filled, an [std::io::ErrorKind::UnexpectedEof] error is returned.
    fn read_i64_slice(self: Pin<&mut Self>, buffer: &mut [i64]) -> Pin<Box<dyn Future<Output = IoResult<()>>>> {
        Box::pin(async move {
            // Safety: data is valid and aligned because we obtained it from a slice of i64s, and the length
            // is determined by the length of the slice. Rust handles the lifetime of the buffer so we know
            // that nobody else is accessing the buffer while we are.
            let mut byte_buffer =
                unsafe { from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, buffer.len() * size_of::<i64>()) };
            self.read_bytes(&mut byte_buffer).await?;
            for i in buffer.iter_mut() {
                *i = i64::from_le_bytes(i.to_le_bytes());
            }
            Ok(())
        })
    }

    /// Reads enough i32s to fill the slice exactly. If the end of the stream is reached before the
    /// slice is filled, an [std::io::ErrorKind::UnexpectedEof] error is returned.
    fn read_i32_slice(self: Pin<&mut Self>, buffer: &mut [i32]) -> Pin<Box<dyn Future<Output = IoResult<()>>>> {
        Box::pin(async move {
            // Safety: data is valid and aligned because we obtained it from a slice of i64s, and the length
            // is determined by the length of the slice. Rust handles the lifetime of the buffer so we know
            // that nobody else is accessing the buffer while we are.
            let mut byte_buffer =
                unsafe { from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, buffer.len() * size_of::<i32>()) };
            self.read_bytes(&mut byte_buffer).await?;
            for i in buffer.iter_mut() {
                *i = i32::from_le_bytes(i.to_le_bytes());
            }
            Ok(())
        })
    }

    /// Reads enough f32s to fill the slice exactly. If the end of the stream is reached before the
    /// slice is filled, an [std::io::ErrorKind::UnexpectedEof] error is returned.
    fn read_f32_slice(self: Pin<&mut Self>, buffer: &mut [f32]) -> Pin<Box<dyn Future<Output = IoResult<()>>>> {
        Box::pin(async move {
            // Safety: data is valid and aligned because we obtained it from a slice of i64s, and the length
            // is determined by the length of the slice. Rust handles the lifetime of the buffer so we know
            // that nobody else is accessing the buffer while we are.
            let mut byte_buffer =
                unsafe { from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, buffer.len() * size_of::<f32>()) };
            self.read_bytes(&mut byte_buffer).await?;
            for i in buffer.iter_mut() {
                *i = f32::from_le_bytes(i.to_le_bytes());
            }
            Ok(())
        })
    }

    /// Reads an i32 stored in variable-length format. This reads between one and five bytes.
    /// Smaller values take fewer bytes. Negative numbers are supported, but should be avoided.
    ///
    /// The format is described in [crate::store::data_output::DataOutput::write_vint].
    fn read_vi32(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<i32>>>> {
        Box::pin(async move {
            let mut b = self.read_byte().await?;
            let mut i = (b & 0x7F) as i32;
            let mut shift = 7;
            while (b & 0x80) != 0 {
                b = self.read_byte().await?;
                i |= ((b & 0x7F) as i32) << shift;
                shift += 7;
            }
            Ok(i)
        })
    }

    /// Reads an i64 stored in variable-length format. This reads between one and nine bytes.
    /// Smaller values take fewer bytes. Negative numbers are supported, but should be avoided.
    ///
    /// The format is described in [crate::store::data_output::DataOutput::write_vint].
    fn read_vi64(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<i64>>>> {
        Box::pin(async move {
            let mut b = self.read_byte().await?;
            let mut i = (b & 0x7F) as i64;
            let mut shift = 7;
            while (b & 0x80) != 0 {
                b = self.read_byte().await?;
                i |= ((b & 0x7F) as i64) << shift;
                shift += 7;
            }
            Ok(i)
        })
    }

    /// Reads a zig-zag encoded i64 variable-length integer. Reads between one and ten bytes.
    ///
    /// The format is described in [crate::store::data_output::DataOutput::write_z64].
    fn read_z64(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<i64>>>> {
        Box::pin(async move {
            let v = self.read_vi64().await?;
            Ok(v.zig_zag_decode())
        })
    }

    /// Reads a string.
    ///
    /// See [crate::store::data_output::DataOutput::write_string].
    fn read_string(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<String>>>> {
        Box::pin(async move {
            let length = self.read_vi32().await?;
            let mut buffer = vec![0u8; length as usize];
            self.read_bytes(&mut buffer).await?;
            String::from_utf8(buffer).map_err(|_| IoError::new(IoErrorKind::InvalidData, "Invalid UTF-8 string"))
        })
    }

    /// Reads a map of strings to strings, previously written with
    /// [crate::store::data_output::DataOutput::write_string_map].
    fn read_map_of_strings(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<HashMap<String, String>>>>> {
        Box::pin(async move {
            let length = self.read_vi32().await?;
            if length == 0 {
                Ok(HashMap::new())
            } else {
                let mut map = HashMap::with_capacity(length as usize);
                for _ in 0..length {
                    let key = self.read_string().await?;
                    let value = self.read_string().await?;
                    map.insert(key, value);
                }
                Ok(map)
            }
        })
    }

    /// Reads a set of strings, previously written with
    /// [crate::store::data_output::DataOutput::write_string_set].
    fn read_set_of_strings(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<HashSet<String>>>>> {
        Box::pin(async move {
            let length = self.read_vi32().await?;
            if length == 0 {
                Ok(HashSet::new())
            } else {
                let mut set = HashSet::with_capacity(length as usize);
                for _ in 0..length {
                    set.insert(self.read_string().await?);
                }
                Ok(set)
            }
        })
    }

    /// Skip over `num_bytes` bytes. This method may skip bytes in whatever way is most optimal, and may not
    /// have the same behavior as reading the skipped bytes.
    fn skip_bytes(self: Pin<&mut Self>, num_bytes: usize) -> Pin<Box<dyn Future<Output = IoResult<()>>>>;
}
