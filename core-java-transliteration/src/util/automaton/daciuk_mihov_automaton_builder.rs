//! Builds a minimal, deterministic {@link Automaton} that accepts a set of strings. The algorithm
//! requires sorted input data, but is very fast (nearly linear with the input size).
use crate::util::automaton::{automaton::Automaton};

/// This builder rejects terms that are more than 1k chars long since it then uses recursion based
/// on the length of the string, which might cause stack overflows.
pub const MAX_TERM_LENGTH: usize = 1_000;

/// Build a minimal, deterministic automaton from a sorted list of `&str` representing
/// strings in UTF-8. These strings must be binary-sorted.
   
pub fn build(input: &[&str]) -> Automaton {
    todo!()
}
