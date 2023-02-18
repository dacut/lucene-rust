use {
    crate::{
        store::data_input::DataInput,
        util::packed::{
            bulk_operation::{new_bulk_operation, BulkOperation},
            packed_ints::{Format, ReaderIterator},
        },
    },
    pin_project::pin_project,
    std::{
        cmp::min,
        future::Future,
        io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult},
        pin::Pin,
    },
};

#[derive(Debug)]
#[pin_project]
pub struct PackedReaderIterator<R> {
    // From ReaderIteratorImpl
    #[pin]
    r#in: R,
    bits_per_value: u32,
    value_count: u32,

    // From PackedReaderIterator
    packed_ints_version: u32,
    format: Format,
    bulk_operation: Box<dyn BulkOperation>,
    next_values: Vec<i64>,
    iterations: u32,
    position: i64,
}

impl<R> PackedReaderIterator<R> {
    pub fn new(
        format: Format,
        packed_ints_version: u32,
        value_count: u32,
        bits_per_value: u32,
        r#in: R,
        mem: usize,
    ) -> Self {
        let bulk_operation = new_bulk_operation(format, bits_per_value);
        let iterations = bulk_operation.compute_iterations(value_count, mem);

        Self {
            r#in,
            bits_per_value,
            value_count,
            packed_ints_version,
            format,
            bulk_operation,
            next_values: Vec::new(),
            iterations,
            position: -1,
        }
    }
}

impl<R> ReaderIterator for PackedReaderIterator<R>
where
    R: DataInput,
{
    fn next(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<i64>>>> {
        // From ReaderIteratorImpl
        Box::pin(async move {
            let values = self.next_range(1).await?;
            if values.is_empty() {
                Err(IoError::new(IoErrorKind::UnexpectedEof, "no more values"))
            } else {
                Ok(values[0])
            }
        })
    }

    fn next_range(self: Pin<&mut Self>, count: usize) -> Pin<Box<dyn Future<Output = IoResult<Vec<i64>>>>> {
        let this = self.project();
        Box::pin(async move {
            let remaining = *this.value_count as i64 - *this.position - 1;
            if remaining <= 0 {
                Err(IoError::new(IoErrorKind::UnexpectedEof, "no more values"))
            } else {
                let remaining = remaining as u64;
                let count = min(remaining as usize, count);

                // Move as many values from the next_values to the result as possible.
                let split = min(count, this.next_values.len());
                let tail = this.next_values.split_off(split);
                let mut result = *this.next_values;
                *this.next_values = tail;

                // Read more values if needed.
                if result.len() < count {
                    assert!(this.next_values.is_empty());

                    // Read data into a buffer. First compute the correct size for the buffer since DataInput
                    // will fail if it doesn't read *exactly* the number of bytes requested.
                    let remaining_blocks =
                        this.format.byte_count(*this.packed_ints_version, remaining as u32, *this.bits_per_value);
                    let block_buffer_size =
                        *this.iterations * BulkOperation::byte_block_count(this.bulk_operation.as_ref());
                    let blocks_to_read = min(remaining_blocks, block_buffer_size as usize);
                    let mut next_blocks = vec![0; blocks_to_read];
                    this.r#in.read_bytes(&mut next_blocks).await?;

                    // Decode the data into the next_values buffer.
                    this.next_values.resize(remaining as usize, 0);
                    this.bulk_operation.decode_u8_to_i64(&next_blocks, this.next_values, *this.iterations)?;

                    // Append the first values from the next_values buffer to the result.
                    let split = min(count - result.len(), this.next_values.len());
                    let tail = this.next_values.split_off(split);
                    result.extend_from_slice(&*this.next_values);
                    *this.next_values = tail;
                }

                Ok(result)
            }
        })
    }

    fn get_bits_per_value(&self) -> u32 {
        // From ReaderIteratorImpl
        self.bits_per_value
    }

    fn size(&self) -> usize {
        // From ReaderIteratorImpl
        self.value_count as usize
    }

    fn ord(&self) -> i32 {
        self.position as i32
    }
}
