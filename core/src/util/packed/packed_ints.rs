//! Simplistic compression for array of `u64` values. Each value is >= 0 and <=
//!  a specified maximum value. The values are stored as packed ints, with each value consuming a
//! fixed number of bits.

use {
    crate::{
        store::data_input::DataInput,
        util::packed::{
            bulk_operation::{new_decoder, new_encoder},
            packed64::Packed64,
            packed64_single_block,
            packed_reader_iterator::PackedReaderIterator,
            packed_writer::PackedWriter,
        },
    },
    std::{
        cmp::{max, min},
        error::Error,
        fmt::{Debug, Display, Formatter, Result as FmtResult},
        future::Future,
        io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult},
        pin::Pin,
    },
    tokio::io::AsyncWrite,
};

/// At most 700% memory overhead, always select a direct implementation.
pub const FASTEST: f32 = 7.0_f32;

/// At most 50% memory overhead, always select a reasonably fast implementation.
pub const FAST: f32 = 0.5_f32;

/// At most 25% memory overhead.
pub const DEFAULT: f32 = 0.25_f32;

/// No memory overhead at all, but the returned implementation may be slow.
pub const COMPACT: f32 = 0.0_f32;

/// Default amount of memory to use for bulk operations.
pub const DEFAULT_BUFFER_SIZE: usize = 1024; // 1K

pub const CODEC_NAME: &str = "PackedInts";

pub const VERSION_MONOTONIC_WITHOUT_ZIGZAG: u32 = 2;

pub const VERSION_START: u32 = VERSION_MONOTONIC_WITHOUT_ZIGZAG;

pub const VERSION_CURRENT: u32 = VERSION_MONOTONIC_WITHOUT_ZIGZAG;

/// A format to write packed ints.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Format {
    /// Compact format, all bits are written contiguously.
    Packed,

    /// A format that may insert padding bits to improve encoding and decoding speed. Since this
    /// format doesn't support all possible bits per value, you should never use it directly, but
    /// rather use [PackedInts::fastest_format_and_bits] to find the format that
    /// best suits your needs.
    #[deprecated(note = "Use Packed instead")]
    PackedSingleBlock,
}

impl Format {
    /// Get a format according to its id.
    pub fn from_id(id: u32) -> Option<Self> {
        match id {
            0 => Some(Format::Packed),
            #[allow(deprecated)]
            1 => Some(Format::PackedSingleBlock),
            _ => None,
        }
    }

    /// Returns the id of the format.
    pub fn get_id(&self) -> u32 {
        match self {
            Format::Packed => 0,
            #[allow(deprecated)]
            Format::PackedSingleBlock => 1,
        }
    }

    /// Computes how many byte blocks are needed to store `values` values of size `bits_per_value`.
    pub fn byte_count(&self, packed_ints_version: u32, value_count: u32, bits_per_value: u32) -> usize {
        assert!(bits_per_value <= 64, "bits_per_value must be <= 64: {bits_per_value}");

        // assume long-aligned
        match self {
            Self::Packed => {
                let bits = value_count * bits_per_value;
                let bytes = bits / 8;
                if bits % 8 == 0 {
                    bytes as usize
                } else {
                    bytes as usize + 1
                }
            }
            _ => 8 * self.long_count(packed_ints_version, value_count, bits_per_value),
        }
    }

    /// Computes how many `u64` blocks are needed to store `values` values of size `bits_per_value`.
    pub fn long_count(&self, packed_ints_version: u32, value_count: u32, bits_per_value: u32) -> usize {
        match self {
            #[allow(deprecated)]
            Self::PackedSingleBlock => {
                let values_per_block = 64 / bits_per_value;
                let blocks = value_count / values_per_block;
                if value_count % values_per_block == 0 {
                    blocks as usize
                } else {
                    blocks as usize + 1
                }
            }
            _ => {
                assert!(bits_per_value <= 64, "bits_per_value must be <= 64: {bits_per_value}");
                let byte_count = self.byte_count(packed_ints_version, value_count, bits_per_value);
                assert!(byte_count < 8 * i32::MAX as usize);

                if byte_count % 8 == 0 {
                    byte_count / 8
                } else {
                    byte_count / 8 + 1
                }
            }
        }
    }

    /// Tests whether the provided number of bits per value is supported by the format.
    pub fn is_supported(&self, bits_per_value: u32) -> bool {
        match self {
            #[allow(deprecated)]
            Self::PackedSingleBlock => packed64_single_block::is_supported(bits_per_value),
            _ => bits_per_value >= 1 && bits_per_value <= 64,
        }
    }

    /// Returns the overhead per value, in bits.
    pub fn overhead_per_value(&self, bits_per_value: u32) -> f32 {
        assert!(self.is_supported(bits_per_value));
        match self {
            #[allow(deprecated)]
            Self::PackedSingleBlock => {
                let values_per_block = 64 / bits_per_value;
                let overhead = 64 % bits_per_value;
                overhead as f32 / values_per_block as f32
            }
            _ => 0.0,
        }
    }

    /// Returns the overhead ratio (`overhead per value / bits per value`).
    pub fn overhead_ratio(&self, bits_per_value: u32) -> f32 {
        assert!(self.is_supported(bits_per_value));
        self.overhead_per_value(bits_per_value) / bits_per_value as f32
    }
}

/// Simple class that holds a format and a number of bits per value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FormatAndBits {
    /// The format.
    pub format: Format,

    /// The number of bits per value.
    pub bits_per_value: u32,
}

impl FormatAndBits {
    pub fn new(format: Format, bits_per_value: u32) -> Self {
        Self {
            format,
            bits_per_value,
        }
    }

    /// Try to find the `Format` and number of bits per value that would restore from disk the
    /// fastest reader whose overhead is less than `acceptable_overhead_ratio`.
    ///
    /// The acceptable_overhead_ratio< parameter makes sense for random-access readers.
    /// In case you only plan to perform sequential access on this stream later on, you
    /// should probably use [Compact].
    ///
    /// If you don't know how many values you are going to write, use `value_count = None`.
    pub fn fastest_format_and_bits(
        value_count: Option<u32>,
        bits_per_value: u32,
        acceptable_overhead_ratio: f32,
    ) -> Self {
        let value_count = value_count.unwrap_or(u32::MAX);

        let acceptable_overhead_ratio = acceptable_overhead_ratio.max(COMPACT);
        let acceptable_overhead_ratio = acceptable_overhead_ratio.min(FASTEST);
        let acceptable_overhead_per_value = acceptable_overhead_ratio * bits_per_value as f32;
        let max_bits_per_value = bits_per_value + acceptable_overhead_per_value as u32;

        // rounded number of bits per value are usually the fastest.
        let actual_bits_per_value = if bits_per_value <= 8 && max_bits_per_value >= 8 {
            8
        } else if bits_per_value <= 16 && max_bits_per_value >= 16 {
            16
        } else if bits_per_value <= 32 && max_bits_per_value >= 32 {
            32
        } else if bits_per_value <= 64 && max_bits_per_value >= 64 {
            64
        } else {
            bits_per_value
        };

        Self {
            format: Format::Packed,
            bits_per_value: actual_bits_per_value,
        }
    }
}

/// A decoder for packed integers.
pub trait Decoder: Debug {
    /// The minimum number of long blocks to encode in a single iteration, when using long encoding.
    fn long_block_count(&self) -> u32;

    /// The number of values that can be stored in [Decoder::long_block_count] long blocks.
    fn long_value_count(&self) -> u32;

    /// The minimum number of byte blocks to encode in a single iteration, when using byte encoding.
    fn byte_block_count(&self) -> u32;

    /// The number of values that can be stored in [Decoder::byte_block_count] byte blocks.
    fn byte_value_count(&self) -> u32;

    /// Read `iterations * block_count()` blocks from `blocks`, decode them and
    /// write `iterations * value_count()` values into `values`.
    ///
    /// # Parameters
    /// `blocks`: the long blocks that hold packed integer values
    /// `values`: values the values buffer
    /// `iterations` controls how much data to decode
    fn decode_u64_to_i64(&mut self, blocks: &[u64], values: &mut [i64], iterations: u32) -> IoResult<()>;

    /// Read `8 * iterations * block_count()` blocks from `blocks`, decode them and
    /// write `iterations * value_count()` values into `values`.
    ///
    /// # Parameters
    /// `blocks`: the long blocks that hold packed integer values
    /// `values`: values the values buffer
    /// `iterations` controls how much data to decode
    fn decode_u8_to_i64(&mut self, blocks: &[u8], values: &mut [i64], iterations: u32) -> IoResult<()>;

    /// Read `iterations * block_count()` blocks from `blocks`, decode them and
    /// write `iterations * value_count()` values into `values`.
    ///
    /// # Parameters
    /// `blocks`: the long blocks that hold packed integer values
    /// `values`: values the values buffer
    /// `iterations` controls how much data to decode
    fn decode_u64_to_i32(&mut self, blocks: &[u64], values: &mut [i32], iterations: u32) -> IoResult<()>;

    /// Read `8 * iterations * block_count()` blocks from `blocks`, decode them and
    /// write `iterations * value_count()` values into `values`.
    ///
    /// # Parameters
    /// `blocks`: the long blocks that hold packed integer values
    /// `values`: values the values buffer
    /// `iterations` controls how much data to decode
    fn decode_u8_to_i32(&mut self, blocks: &[u8], values: &mut [i32], iterations: u32) -> IoResult<()>;
}

/// An encoder for packed integers
pub trait Encoder: Debug {
    /// The minimum number of long blocks to encode in a single iteration, when using long encoding.
    fn long_block_count(&self) -> u32;

    /// The number of values that can be stored in [Encoder::long_block_count] long blocks.
    fn long_value_count(&self) -> u32;

    /// The minimum number of byte blocks to encode in a single iteration, when using byte encoding.
    fn byte_block_count(&self) -> u32;

    /// The number of values that can be stored in {@link #byteBlockCount()} byte blocks.
    fn byte_value_count(&self) -> u32;

    /// Read `iterations * value_count()` values from `values`, encode them and
    /// write `iterations * block_bount()` blocks into `blocks`.
    ///
    /// # Parameters
    /// `blocks`: the long blocks that hold packed integer values
    /// `values`: values the values buffer
    /// `iterations` controls how much data to decode
    fn encode_i64_to_u64(&mut self, values: &[i64], blocks: &mut [u64], iterations: u32) -> IoResult<()>;

    /// Read `iterations * value_count()` values from `values`, encode them and
    /// write `8 * iterations * block_bount()` blocks into `blocks`.
    ///
    /// # Parameters
    /// `blocks`: the long blocks that hold packed integer values
    /// `values`: values the values buffer
    /// `iterations` controls how much data to decode
    fn encode_i64_to_u8(&mut self, values: &[i64], blocks: &mut [u8], iterations: u32) -> IoResult<()>;

    /// Read `iterations * value_count()` values from `values`, encode them and
    /// write `iterations * block_bount()` blocks into `blocks`.
    ///
    /// # Parameters
    /// `blocks`: the long blocks that hold packed integer values
    /// `values`: values the values buffer
    /// `iterations` controls how much data to decode
    fn encode_i32_to_u64(&mut self, values: &[i32], blocks: &mut [u64], iterations: u32) -> IoResult<()>;

    /// Read `iterations * value_count()` values from `values`, encode them and
    /// write `8 * iterations * block_bount()` blocks into `blocks`.
    ///
    /// # Parameters
    /// `blocks`: the long blocks that hold packed integer values
    /// `values`: values the values buffer
    /// `iterations` controls how much data to decode
    fn encode_i32_to_u8(&mut self, values: &[i32], blocks: &mut [u8], iterations: u32) -> IoResult<()>;
}

/// A read-only random access array of positive integers.
pub trait Reader: Debug {
    /// Get the long at the given index. Behavior is undefined for out-of-range indices.
    fn get(&self, index: usize) -> i64;

    /// Bulk get: read at least one and at most `arr.len()` i64 values starting from `index`
    /// into `arr` and return the actual number of values that have been read.
    fn get_range(&self, index: usize, arr: &mut [i64]) -> usize {
        let size = self.size();
        assert!(index < size);
        let len = arr.len();
        let to_get = min(size - index, len);
        for i in 0..to_get {
            arr[i] = self.get(index + i);
        }

        to_get
    }

    /// Returns the number of values.
    fn size(&self) -> usize;
}

/// Run-once iterator interface, to decode previously saved PackedInts.
pub trait ReaderIterator {
    /// Returns the next value.
    fn next(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<i64>>>>;

    /// Returns at least 1 and at most `count` next values.
    fn next_range(self: Pin<&mut Self>, count: usize) -> Pin<Box<dyn Future<Output = IoResult<Vec<i64>>>>>;

    /// Returns the number of bits per value.
    fn get_bits_per_value(&self) -> u32;

    /// Returns the number of values.
    fn size(&self) -> usize;

    /// Returns the current position.
    fn ord(&self) -> i32;
}

/// A packed integer array that can be modified.
pub trait Mutable: Reader {
    /// Returns the number of bits used to store any given value.
    ///
    /// # Note
    /// This does not imply that memory usage is `bitsPerValue * values` as implementations are free to use
    /// non-space-optimal packing of bits.
    fn get_bits_per_value(&self) -> u32;

    /// Set the value at the given index in the array.
    fn set(&mut self, index: usize, value: i64);

    /// Bulk set: set at least one and at most `arr.len()` `i64` values into this mutable, starting at
    /// `index`. Returns the actual number of values that have been set.
    fn set_range(&mut self, index: usize, arr: &[i64]) -> usize {
        let size = self.size();
        assert!(index < size);
        let len = arr.len();
        let to_set = min(size - index, len);
        for i in 0..to_set {
            self.set(index + i, arr[i]);
        }

        to_set
    }

    /// Fill the mutable from `from_index` (inclusive) to `to_index` (exclusive) with `val`.
    fn fill(&mut self, from_index: usize, to_index: usize, val: i64) {
        assert!(from_index <= to_index);
        assert!(to_index <= self.size());
        for i in from_index..to_index {
            self.set(i, val);
        }
    }

    /// Sets all values to 0.
    fn clear(&mut self) {
        self.fill(0, self.size(), 0);
    }

    // Hack to allow trait upcasing.
    fn into_reader(self: Box<Self>) -> Box<dyn Reader>;
}

/// A [Reader] which has all its values equal to 0 (`bits_per_value = 0`).
#[derive(Debug)]
pub struct NullReader {
    value_count: u32,
}

impl NullReader {
    pub fn new(value_count: u32) -> Self {
        NullReader {
            value_count,
        }
    }
}

impl Reader for NullReader {
    fn get(&self, _index: usize) -> i64 {
        0
    }

    fn get_range(&self, index: usize, arr: &mut [i64]) -> usize {
        assert!(index < self.value_count as usize);
        let len = min(arr.len(), self.value_count as usize - index);
        for i in 0..len {
            arr[i] = 0;
        }
        len
    }

    fn size(&self) -> usize {
        self.value_count as usize
    }
}

/// A write-once Writer.
pub trait Writer: Debug {
    /// The format used to serialize values.
    fn get_format(&self) -> Format;

    /// Add a value to the stream.
    fn add(self: Pin<&mut Self>, v: i64) -> Pin<Box<dyn Future<Output = IoResult<()>>>>;

    /// The number of bits per value.
    fn bits_per_value(&self) -> u32;

    /// Perform end-of-stream operations.
    fn finish(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<()>>>>;

    /// Returns the current ord in the stream (number of values that have been written so far minus
    /// one).
    fn ord(&self) -> i32;
}

/// Get a [Decoder].
///
/// # Parameters
/// * `format`: the format used to store packed ints
/// * `version` the compatibility version
/// * `bits_per_value`: the number of bits per value
pub fn get_decoder(format: Format, version: u32, bits_per_value: u32) -> Box<dyn Decoder> {
    new_decoder(format, bits_per_value)
}

/// Get an [Encoder].
///
/// # Parameters
/// * `format`: the format used to store packed ints
/// * `version` the compatibility version
/// * `bits_per_value`: the number of bits per value
pub fn get_encoder(format: Format, version: u32, bits_per_value: u32) -> Box<dyn Encoder> {
    new_encoder(format, bits_per_value)
}

/// Expert: Restore a [ReaderIterator] from a stream without reading metadata at the
/// beginning of the stream. This method is useful to restore data from streams which have been
/// created using [PackedInts::get_writer_no_header].
///
/// # Parameters
/// * `in`: the stream to read data from, positioned at the beginning of the packed values
/// * `format`: the format used to serialize
/// * `version`: the version used to serialize the data
/// * `value_count`: how many values the stream holds
/// * `bits_per_value`: the number of bits per value
/// * `mem`: how much memory the iterator is allowed to use to read-ahead (likely to speed up
///   iteration)
///
/// See also: [PackedInts::get_write_no_header]
pub fn get_reader_iterator_no_header<R>(
    r#in: R,
    format: Format,
    version: u32,
    value_count: u32,
    bits_per_value: u32,
    mem: usize,
) -> Result<Pin<Box<dyn ReaderIterator>>, Box<dyn Error + Send + Sync + 'static>>
where
    R: DataInput,
{
    check_version(version)?;
    let pri = PackedReaderIterator::new(format, version, value_count, bits_per_value, r#in, mem);
    let result: Pin<Box<dyn ReaderIterator>> = Box::pin(pri);
    Ok(result)
}

/// Expert: Create a packed integer array writer for the given output, format, value count, and
/// number of bits per value.
///
/// The resulting stream will be long-aligned. This means that depending on the format which is
/// used, up to 63 bits will be wasted. An easy way to make sure that no space is lost is to always
/// use a `valueCount` that is a multiple of 64.
///
/// This method does not write any metadata to the stream, meaning that it is your
/// responsibility to store it somewhere else in order to be able to recover data from the stream
/// later on:
///
/// * `format` (using [Format::get_id],
/// * `value_count`,
/// * `bits_per_value`,
/// * [VERSION_CURRENT],
///
/// It is possible to start writing values without knowing how many of them you are actually
/// going to write. To do this, just pass `None` as value_count. On the other
/// hand, for any `Some` value of value_count, the returned writer will make sure
/// that you don't write more values than expected and pad the end of stream with zeros in case you
/// have written less than value_count when calling [Writer::finish].
///
/// The `mem` parameter lets you control how much memory can be used to buffer
/// changes in memory before flushing to disk. High values of `mem` are likely to
/// improve throughput. On the other hand, if speed is not that important to you, a value of `0`
///  will use as little memory as possible and should already offer reasonable throughput.
///
/// # Parameters
/// * `out`: the data output
/// * `format`: the format to use to serialize the values
/// * `value_count`: the number of values
/// * `bits_per_value`: the number of bits per value
/// * `mem`: how much memory (in bytes) can be used to speed up serialization
///
/// See [PackedInts::get_reader_iterator_no_header]
pub fn get_writer_no_header<W>(
    out: Pin<&mut W>,
    format: Format,
    value_count: Option<u32>,
    bits_per_value: u32,
    mem: usize,
) -> PackedWriter
where
    W: AsyncWrite,
{
    PackedWriter::new(format, out, value_count, bits_per_value, mem)
}

/// Create a packed integer array with the given amount of values initialized to 0. the valueCount
/// and the bitsPerValue cannot be changed after creation. All Mutables known by this factory are
/// kept fully in RAM.
///
/// Positive values of `acceptable_overhead_ratio will` trade space for speed by
/// selecting a faster but potentially less memory-efficient implementation. An
/// `acceptable_overhead_ratio` of [COMPACT] will make sure that the most
/// memory-efficient implementation is selected whereas [FASTEST] will make sure
/// that the fastest implementation is selected.
///
/// # Parameters
/// * `value_count`: the number of elements
/// * `bits_per_value`: the number of bits available for any given value
/// * `acceptable_overhead_ratio` an acceptable overhead ratio per value
pub fn get_mutable(value_count: u32, bits_per_value: u32, acceptable_overhead_ratio: f32) -> Box<dyn Mutable> {
    let format_and_bits =
        FormatAndBits::fastest_format_and_bits(Some(value_count), bits_per_value, acceptable_overhead_ratio);
    get_mutable_for_format_and_bits(value_count, format_and_bits.bits_per_value, format_and_bits.format)
}

/// Same as [Writer::get_mutable] with a pre-computed number of bits per value and
/// format.
pub fn get_mutable_for_format_and_bits(value_count: u32, bits_per_value: u32, format: Format) -> Box<dyn Mutable> {
    match format {
        #[allow(deprecated)]
        Format::PackedSingleBlock => packed64_single_block::new_mutable(value_count, bits_per_value).unwrap(),
        Format::Packed => Box::new(Packed64::new(value_count, bits_per_value)),
    }
}

/// Check the validity of a version number.
pub fn check_version(version: u32) -> Result<(), VersionError> {
    if version < VERSION_START {
        Err(VersionError::TooOld(version))
    } else if version > VERSION_CURRENT {
        Err(VersionError::TooNew(version))
    } else {
        Ok(())
    }
}

#[derive(Debug)]
pub enum VersionError {
    TooOld(u32),
    TooNew(u32),
}

impl Display for VersionError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            VersionError::TooOld(v) => write!(f, "Version is too old, should be at least {VERSION_START} (got {v})"),
            VersionError::TooNew(v) => write!(f, "Version is too new, should be at mod {VERSION_CURRENT} (got {v})"),
        }
    }
}

impl Error for VersionError {}

/// Returns how many bits are required to hold values up to and including maxValue
///
/// # Note
/// This method returns at least 1.
///
/// # Parameters
/// * `max_value`: the maximum value that should be representable.
///
/// # Returns
/// The amount of bits needed to represent values from 0 to maxValue.
pub fn bits_required(max_value: i64) -> u32 {
    assert!(max_value >= 0, "max_value must be non-negative (got: {max_value})");

    unsigned_bits_required(max_value)
}

/// Returns how many bits are required to store `bits`, interpreted as an unsigned
/// value.
///
/// #Note
/// This method returns at least 1.
pub fn unsigned_bits_required(bits: i64) -> u32 {
    max(1, 64 - bits.leading_zeros())
}

/// Calculates the maximum i64 that can be expressed with the given number of bits.
///
/// # Parameters
/// * `bits_per_value`: the number of bits available for any given value.
///
/// # Returns
/// The maximum value for the given bits.
pub fn max_value(bits_per_value: u32) -> i64 {
    if bits_per_value == 64 {
        i64::MAX
    } else {
        !(!0 << bits_per_value)
    }
}

//// Check that the block size is a power of 2, in the right bounds, and return its log in base 2.
pub(crate) fn check_block_size(block_size: u32, min_block_size: u32, max_block_size: u32) -> IoResult<u32> {
    if block_size < min_block_size || block_size > max_block_size {
        return Err(IoError::new(
            IoErrorKind::InvalidInput,
            format!("block size must be >= {min_block_size} and <= {max_block_size} (got: {block_size})"),
        ));
    }

    if block_size & (block_size - 1) != 0 {
        return Err(IoError::new(
            IoErrorKind::InvalidInput,
            format!("block size must be a power of 2 (got: {block_size})"),
        ));
    }

    Ok(block_size.trailing_zeros())
}
