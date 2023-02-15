use {
    crate::util::vector_util::{Cosine, DotProduct, DotProductScore, SquareDistance}
};

/// Vector similarity function; used in search to return top K most similar vectors to a target
/// vector. This is a label describing the method used during indexing and searching of the vectors
/// in order to determine the nearest neighbors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VectorSimilarityFunction {
    /// Euclidean distance
    Euclidean,

    /// Dot product.
    ///
    /// # Note
    /// This similarity is intended as an optimized way to perform cosine similarity. In order to use it,
    /// all vectors must be normalized, including both document and query vectors. Using dot product with
    /// vectors that are not normalized can result in errors or poor search results. Floating point vectors
    ///  must be normalized to be of unit length, while byte vectors should simply all have the same norm.
    DotProduct,

    /// Cosine similarity.
    ///
    /// # Note
    /// The preferred way to perform cosine similarity is to normalize all vectors to unit length, and
    /// instead use [VectorSimiliarityFunction::DotProduct]. You should only use this function if you need
    /// to preserve the original vectors and cannot normalize them in advance. The similarity score is
    ///  normalised to assure it is positive.
    Cosine,
}

impl VectorSimilarityFunction {
    pub fn compare_f32(&self, v1: &[f32], v2: &[f32]) -> f32 {
        match self {
            Self::Euclidean => 1.0 / (1.0 + v1.square_distance(v2)),
            Self::DotProduct => v1.dot_product(v2),
            Self::Cosine => v1.cosine(v2),
        }
    }

    pub fn compare_u8(&self, v1: &[u8], v2: &[u8]) -> f32 {
        match self {
            Self::Euclidean => 1.0 / (1.0 + v1.square_distance(v2)),
            Self::DotProduct => v1.dot_product_score(v2),
            Self::Cosine => v1.cosine(v2),
        }
    }
}
