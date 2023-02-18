use {crate::util::automaton::state::State};

/// A runnable automaton accepting byte array as input
pub trait ByteRunnable {
    /// Returns the state obtained by reading the given char from the given state. Returns `None` if not
    /// obtaining any such state.
    ///
    /// # Arguments
    /// * `state`: the last state
    /// * `c`: the input codepoint
    /// 
    /// # Returns
    /// The next state, or `None` if no such transaction
    fn step(&self, state: State, c: char) -> Option<State>;

    /// Returns whether the given state is an accept state for this automaton.
    fn is_accept(&self, state: State) -> bool;

    /// Returns number of states this automaton has, note this may not be an accurate number in case of
    /// NFA
    fn size(&self) -> usize;

    /// Returns true if the given byte array is accepted by this automaton.
    fn run(&self, s: &str) -> bool {
        let mut state = State(0); // Start with the initial state, always 0.
        for c in s.chars() {
            match self.step(state, c) {
                Some(next_state) => state = next_state,
                None => return false,
            }
        }

        self.is_accept(state)
    }
}
