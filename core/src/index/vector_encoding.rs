/// The numeric datatype of the vector values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VectorEncoding {
    /// Encodes vector using 8 bits of precision per sample. Values provided with higher precision (eg:
    /// queries provided as float) *must* be in the range [-128, 127]. NOTE: this can enable
    /// significant storage savings and faster searches, at the cost of some possible loss of
    /// precision.
    Byte,

    /// Encodes vector using 32 bits of precision per sample in IEEE floating point format.
    Float32,
}

impl VectorEncoding {
    pub fn byte_size(&self) -> usize {
        match self {
            VectorEncoding::Byte => 1,
            VectorEncoding::Float32 => 4,
        }
    }
}
