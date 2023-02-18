use {
    std::{
        error::Error,
        fmt::{Display, Formatter, Result as FmtResult},
    }
};

/// This error is returned when determinizing an automaton would require too much work.
#[derive(Debug)]
pub struct TooComplexToDeterminizeError {
    /// The work limit that was exceeded.
    pub determinize_work_limit: usize,
}

impl TooComplexToDeterminizeError {
    /// Create a new `TooComplexToDeterminizeError`.
    pub fn new(determinize_work_limit: usize) -> Self {
        Self { determinize_work_limit }
    }
}

impl Display for TooComplexToDeterminizeError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(
            f,
            "Determinizing this automaton would require more than {} work units",
            self.determinize_work_limit
        )
    }
}

impl Error for TooComplexToDeterminizeError {}