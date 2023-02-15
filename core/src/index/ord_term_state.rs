use crate::index::term_state::TermState;

/// An ordinal based [TermState].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OrdTermState {
    /// Term ordinal, i.e. its position in the full list of sorted terms.
    pub ord: i64,
}

impl OrdTermState {
    pub fn new(ord: i64) -> Self {
        Self { ord }
    }
}

impl TermState for OrdTermState {}
