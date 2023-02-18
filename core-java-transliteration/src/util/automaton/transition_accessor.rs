use {
    crate::util::automaton::{state::State, transition::Transition},
};

/// Interface accessing the transitions of an automaton
pub trait TransitionAccessor {
    /// Initialize the provided Transition to iterate through all transitions leaving the specified
    /// state. You must call [::get_next_tansition] to get each transition. Returns the number of
    /// transitions leaving this state.
    fn init_transition(&self, state: State, t: &mut Transition) -> usize;
  
    /// Iterate to the next transition after the provided one
    fn get_next_transition(&mut self, t: &mut Transition);
  
    /// How many transitions this state has.
    fn get_num_transitions(&self, state: State) -> usize;
  
    /// Update the [Transition] with the index'th transition leaving the specified state.
    fn get_transition(&self, state: State, index: usize, t: &mut Transition);
}
  