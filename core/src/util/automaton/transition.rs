use {
    crate::util::automaton::state::State,
    std::fmt::{Display, Formatter, Result as FmtResult},
};

/// Holds one transition from an [Automaton]. This is typically used temporarily when iterating
/// through transitions by invoking [Automaton::init_transition] and [Automaton::get_next_transition].
#[derive(Debug)]
pub struct Transition {
    /// Source state.
    pub source: State,

    /// Destination state.
    pub dest: Option<State>,

    /// Minimum accepted codepoint (inclusive).
    pub min: u32,

    /// Maximum accepted codepoint (inclusive).
    pub max: u32,

    /// Remembers where we are in the iteration; init to `None` to return an error if 
    /// [Automaton::get_next_transition] is called before [Automaton::init_transition].
    pub transition_upto: Option<u32>,
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            source: State(0),
            dest: None,
            min: 0,
            max: 0,
            transition_upto: None,
        }
    }
}

impl Display for Transition {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let source = self.source.0.to_string();
        let dest = match self.dest {
            None => "None".to_string(),
            Some(dest) => dest.0.to_string(),
        };

        write!(f, "{source} -> {dest} {}-{}", self.min, self.max)
    }
}
