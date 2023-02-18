use crate::util::automaton::{byte_runnable::ByteRunnable, state::State};

#[derive(Debug)]
pub struct NFARunAutomaton {
    // TODO: implement
}

impl ByteRunnable for NFARunAutomaton {
    fn step(&self, state: State, c: char) -> Option<State> {
        todo!()
    }

    /// Returns whether the given state is an accept state for this automaton.
    fn is_accept(&self, state: State) -> bool {
        todo!()
    }

    /// Returns number of states this automaton has, note this may not be an accurate number in case of
    /// NFA
    fn size(&self) -> usize {
        todo!()
    }
}