use std::{
    fmt::Debug,
    future::{ready, Future},
    io::Result as IoResult,
    pin::Pin,
};

/// Allows access to the score of a Query
pub trait Scorable: Debug {
    /// Returns the score of the current document matching the query.
    fn score(&self) -> IoResult<f32>;

    /// Returns the smoothing score of the current document matching the query. This score is used when
    /// the query/term does not appear in the document, and behaves like an idf. The smoothing score is
    /// particularly important when the Scorer returns a product of probabilities so that the document
    /// score does not go to zero when one probability is zero. This can return `None` or a smoothing score.
    ///
    /// Smoothing scores are described in many papers, including: Metzler, D. and Croft, W. B. ,
    /// "Combining the Language Model and Inference Network Approaches to Retrieval," Information
    /// Processing and Management Special Issue on Bayesian Networks and Information Retrieval, 40(5),
    /// pp.735-750.
    fn smoothing_score(self: Pin<&mut Self>, doc_id: i32) -> Pin<Box<dyn Future<Output = IoResult<Option<f32>>>>> {
        Box::pin(ready(Ok(None)))
    }

    /// Returns the doc ID that is currently being scored.
    fn doc_id(&self) -> i32;

    /// Optional method: Tell the scorer that its iterator may safely ignore all documents whose score
    /// is less than the given `min_score`. This is a no-op by default.
    ///
    /// This method may only be called from collectors that use [ScoreMode::TopScores], and
    /// successive calls may only set increasing values of `min_score`.
    fn set_min_competitive_score(&mut self, min_score: f32) -> IoResult<()> {
        Ok(())
    }

    /// Returns child sub-scorers positioned on the current document
    fn get_chidren(&self) -> IoResult<Vec<ChildScorable>> {
        Ok(vec![])
    }
}

/// A child Scorer and its relationship to its parent. The meaning of the relationship depends upon
/// the parent query.
#[derive(Debug)]
pub struct ChildScorable {
    /// Child Scorer. (note this is typically a direct child, and may itself also have children).
    pub child: Box<dyn Scorable>,

    /// An arbitrary string relating this scorer to the parent.
    pub relationship: String,
}

impl ChildScorable {
    /// Creates a new ChildScorer node with the specified relationship.
    ///
    /// The relationship can be any string that makes sense to the parent Scorer.
    pub fn new(child: Box<dyn Scorable>, relationship: &str) -> Self {
        ChildScorable {
            child,
            relationship: relationship.to_string(),
        }
    }
}
