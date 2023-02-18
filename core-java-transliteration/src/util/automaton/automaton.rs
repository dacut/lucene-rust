use {
    crate::util::automaton::{state::State, transition::Transition, transition_accessor::TransitionAccessor},
    bitvec::prelude::*,
    std::{collections::BTreeSet, cmp::{Ordering, Ord, PartialOrd}},
};

/// Represents an automaton and all its states and transitions. States are integers and must be
/// created using [Automaton::create_state]. Mark a state as an accept state using [set_accept]. Add
/// transitions using [add_transition]. Each state must have all of its transitions added at
/// once; if this is too restrictive then use [Automaton::Builder] instead. State 0 is always
/// the initial state. Once a state is finished, either because you've starting adding transitions to
/// another state or you call [finish_state], then that states transitions are sorted (first by
/// min, then max, then dest) and reduced (transitions with adjacent labels going to the same dest
/// are combined).
#[derive(Clone, Debug, Default)]
pub struct Automaton {
    // Where we next write to the Vec<> states
    // next_state: u32 -- always states.len() in Rust.

    // Where we next write to in Vec<> transitions
    // next_transition: u32, -- always transitions.len() in Rust.

    /// Current state we are adding transitions to; the caller must add all transitions for this state
    /// before moving onto another state.
    cur_state: Option<u32>,

    /// Index in the transitions array, where this states leaving transitions are stored, or `None` if this
    /// state has not added any transitions yet, followed by number of transitions.
    states: Vec<StateInfo>,

    is_accept: BitVec,

    /// Holds toState, min, max for each transition.
    transitions: Vec<TransitionInfo>,

    /// True if no state has two transitions leaving with the same label.
    deterministic: bool,
}

#[derive(Clone, Copy, Debug)]
struct StateInfo {
    pub transitions_index: Option<u32>,
    pub num_transitions: u32,
}

impl Default for StateInfo {
    fn default() -> Self {
        Self {
            transitions_index: None,
            num_transitions: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct TransitionInfo {
    pub to_state: Option<State>,
    pub min: u32,
    pub max: u32,
}

impl Default for TransitionInfo {
    fn default() -> Self {
        Self {
            to_state: None,
            min: 0,
            max: 0,
        }
    }
}

impl Automaton {
    /// Constructor which creates an automaton with enough space for the given number of states and
    /// transitions.
    pub fn new(num_states: usize, num_transitions: usize) -> Self {
        Self {
            cur_state: None,
            states: vec![StateInfo::default(); num_states],
            is_accept: bitvec![0; num_states],
            transitions: vec![TransitionInfo::default(); num_transitions],
            deterministic: false,
        }
    }

    /// Create a new state.
    pub fn create_state(&mut self) -> State {
        let state: u32 = self.states.len().try_into().unwrap();
        self.states.push(StateInfo::default());
        State(state)
    }

    /// Set or clear this state as an accept state.
    pub fn set_accept(&mut self, state: State, accept: bool) {
        assert!(state.0 < self.get_num_states());
        self.is_accept.set(state.usize(), accept);
    }

    // Sugar to get all transitions for all states. This is object-heavy; it's better to iterate state by state instead.
    pub fn get_sorted_transitions(&self) -> Vec<Vec<Transition>> {
        let num_states = self.get_num_states();
        let mut transitions = Vec::with_capacity(num_states as usize);

        for s in 0..num_states {
            let num_transitions = TransitionAccessor::get_num_transitions(self, State(s));
            let state_transitions = Vec::with_capacity(num_transitions);
            for t in 0..num_transitions {
                let mut transition = Transition::default();
                self.get_transition(State(s), t, &mut transition);
                state_transitions.push(transition);
            }

            transitions.push(state_transitions);
        }

        transitions
    }

    /// Returns the accepted states.
    pub fn get_accept_states(&self) -> &BitVec {
        &self.is_accept
    }

    /// Returns true if this state is an accept state.
    pub fn is_accept(&self, state: State) -> bool {
        assert!(state.0 < self.get_num_states());
        self.is_accept[state.usize()]
    }

    /// Add a new transition with min = max = label.
    #[inline]
    pub fn add_transition(&mut self, source: State, dest: State, label: u32) {
        self.add_transition_range(source, dest, label, label);
    }

    /// Add a new transition with the specified source, dest, min, max.
    pub fn add_transition_range(&mut self, source: State, dest: State, min: u32, max: u32) {
        assert!(source.usize() < self.states.len());
        assert!(dest.usize() < self.states.len());

        let transition_id = self.transitions.len().try_into().unwrap();

        if self.cur_state != Some(source.0) {
            if self.cur_state.is_none() {
                self.finish_current_state();
            }

            // Move to next source.
            self.cur_state = Some(source.0);
            assert!(self.states[source.usize()].transitions_index.is_none());
            assert!(self.states[source.usize()].num_transitions == 0);

            self.states[source.usize()].transitions_index = Some(transition_id);
        }

        self.transitions.push(TransitionInfo {
            to_state: Some(dest),
            min: min as u32,
            max: max as u32,
        });

        // Increment transition count for this state.
        self.states[source.usize()].num_transitions += 1;
    }

    /// Add a (virtual) epsilon transition between source and dest. Dest state must already have all
    /// transitions added because this method simply copies those same transitions over to source.
    pub fn add_epsilon(&mut self, source: State, dest: State) {
        let mut t = Transition {
            source,
            dest: Some(dest),
            min: 0,
            max: 0,
            transition_upto: None,
        };

        let count = self.init_transition(dest, &mut t);
        for i in 0..count {
            self.get_next_transition(&mut t);
            self.add_transition_range(source, t.dest.unwrap(), t.min, t.max);
        }

        if self.is_accept(dest) {
            self.set_accept(source, true);
        }
    }

    /// Copies over all states/transitions from other. The states numbers are sequentially assigned (appended).
    pub fn copy(&mut self, other: &Automaton) {
        // Bulk copy and then fixup the state pointers:
        let state_offset = self.get_num_states();
        let transitions_offset: u32 = self.get_num_transitions().try_into().unwrap();

        self.states.extend_from_slice(&other.states);

        // Offset the transitions from other.
        for &mut state in &mut self.states[state_offset as usize..] {
            if let Some(index) = state.transitions_index.as_mut() {
                *index += transitions_offset;
            }
        }

        // Add accept states from the other automaton, fixed up by state_offset.
        let other_num_states = other.get_num_states();
        let other_accept_states = other.get_accept_states();
        for state in other_accept_states.iter_ones() {
            let state: u32 = state.try_into().unwrap();
            self.set_accept(State(state_offset + state), true);
        }

        // Bulk copy and then fixup dest for each transition.
        for transition in &other.transitions {
            self.transitions.push(TransitionInfo{
                to_state: transition.to_state.map(|s| State(state_offset + s.0)),
                min: transition.min,
                max: transition.max,
            });
        }

        if !other.deterministic {
            self.deterministic = false;
        }
    }

    /// Freezes the last state, sorting and reducing the transitions.
    fn finish_current_state(&mut self) {
        let num_transitions = self.states[self.cur_state.unwrap() as usize].num_transitions;
        assert!(num_transitions > 0);

        let start = self.states[self.cur_state.unwrap() as usize].transitions_index.unwrap();

        self.sort_transitions_dest_min_max(start, start + num_transitions);

        // Reduce any "adjacent" transitions:
        let mut upto: u32 = 0;
        let mut min: Option<u32> = None;
        let mut max: Option<u32> = None;
        let mut dest: Option<State> = None;

        for i in 0..num_transitions {
            let t = &self.transitions[(start + i) as usize];
            let t_dest = t.to_state;
            let t_min = t.min;
            let t_max = t.max;

            if dest == t_dest {
                // Safety: We're violating Unicode here to do this.
                if max.is_some() && t_min < max.unwrap() as u32 + 1 {
                    if t_max > max.unwrap() {
                        max = Some(t_max);
                    }
                } else {
                    if dest.is_some() {
                        let t = &mut self.transitions[(start + upto) as usize];
                        t.to_state = dest;
                        t.min = min.unwrap();
                        t.max = max.unwrap();
                        upto += 1;
                    }

                    min = Some(t_min);
                    max = Some(t_max);
                }
            } else {
                if dest.is_some() {
                    let t = &mut self.transitions[(start + upto) as usize];
                    t.to_state = dest;
                    t.min = min.unwrap();
                    t.max = max.unwrap();
                    upto += 1;
                }

                dest = t_dest;
                min = Some(t_min);
                max = Some(t_max);
            }
        }

        if dest.is_some() {
            // Last transition
            let t = &mut self.transitions[(start + upto) as usize];
            t.to_state = dest;
            t.min = min.unwrap();
            t.max = max.unwrap();
            upto += 1;
        }

        let next_transition = self.transitions.len() - (num_transitions - upto) as usize;
        self.transitions.truncate(next_transition);
        self.states[self.cur_state.unwrap() as usize].num_transitions = upto;

        // Sort transitions by min/max/dest
        self.sort_transitions_min_max_dest(start, start + upto);

        if self.deterministic && upto > 1 {
            let last_max = self.transitions[start as usize].max;
            for i in 1..upto {
                let t = &self.transitions[(start + i) as usize];
                let min = t.min;
                if min <= last_max {
                    self.deterministic = false;
                    break;
                }
                last_max = t.max;
            }
        }
    }

    /// Returns true if this automaton is deterministic (for ever state there is only one transition
    /// for each label).
    #[inline]
    pub fn is_deterministic(&self) -> bool {
        self.deterministic
    }

    /// Finishes the current state; call this once you are done adding transitions for a state. This is
    /// automatically called if you start adding transitions to a new source state, but for the last
    /// state you add you need to this method yourself.
    pub fn finish_state(&mut self) {
        if self.cur_state.is_some() {
            self.finish_current_state();
            self.cur_state = None;
        }
    }

    /// The number of states this automaton has.
    #[inline]
    pub fn get_num_states(&self) -> u32 {
        self.states.len().try_into().unwrap()
    }

    /// The number of transitions this automaton has.
    #[inline]
    pub fn get_num_transitions(&self) -> usize {
        self.transitions.len()
    }

    fn grow_states(&mut self) {
        // No-op in Rust.
    }

    fn grow_transitions(&mut self) {
        // No-op in Rust.
    }

    /// Sorts transitions by dest, ascending, then min label ascending, then max label ascending.
    fn sort_transitions_dest_min_max(&mut self, start: u32, end: u32) {
        let start = start as usize;
        let end = end as usize;
        self.transitions[start..end].sort_by(|a, b| {
            match a.to_state.unwrap().cmp(&b.to_state.unwrap()) {
                Ordering::Equal => {
                    match a.min.cmp(&b.min) {
                        Ordering::Equal => a.max.cmp(&b.max),
                        other => other,
                    }
                }
                other => other,
            }
        })
    }

    /// Sorts transitions by min label, ascending, then max label ascending, then dest ascending.
    fn sort_transitions_min_max_dest(&mut self, start: u32, end: u32) {
        let start = start as usize;
        let end = end as usize;
        self.transitions[start..end].sort_by(|a, b| {
            match a.min.cmp(&b.min) {
                Ordering::Equal => {
                    match a.max.cmp(&b.max) {
                        Ordering::Equal => a.to_state.unwrap().cmp(&b.to_state.unwrap()),
                        other => other,
                    }
                }
                other => other,
            }
        })
    }   

    fn transition_sorted(&self, t: &Transition) -> bool {
        let upto = t.transition_upto.unwrap();
        if upto == self.states[t.source.usize()].transitions_index.unwrap() {
            // Transition isn't initialized yet (this is the first transition); don't check.
            return true;
        }

        let next_dest = self.transitions[upto as usize].to_state.unwrap();
        let next_min = self.transitions[upto as usize].min;
        let next_max = self.transitions[upto as usize].max;

        if next_min > t.min {
            return true;
        }

        if next_min < t.min {
            return false;
        }

        // Min is equal; test max;
        if next_max > t.max {
            return true;
        }

        if next_max < t.max {
            return false;
        }

        // Max is also equal; test dest.
        if next_dest > t.dest.unwrap() {
            return true;
        }

        false
    }

    /// Returns a sorted vec of all interval start points.
    pub fn get_start_points(&self) -> Vec<u32> {
        let mut pointset = BTreeSet::new();
        pointset.insert(0);
        for s in 0..self.states.len() {
            let mut trans = self.states[s as usize].transitions_index.unwrap();
            let mut limit = trans + self.states[(s + 1) as usize].transitions_index.unwrap();

            while trans < limit {
                let min = self.transitions[trans as usize].min;
                let max = self.transitions[trans as usize].max;
                pointset.insert(min);
                if max < char::MAX as u32 {
                    pointset.insert(max + 1);
                }
                trans += 1;
            }
        }
        pointset.into_iter().collect()
    }

    /// Performs lookup in transitions, assuming determinism.
    /// 
    /// # Arguments
    /// `state`: The state to start the lookup from.
    /// `label`: The codepoint to look up.
    /// 
    /// # Returns
    /// The destination state, or `None` if no matching outgoing transition.
    pub fn step(&self, state: State, label: u32) -> Option<State> {
        self.next_state_update_transition(state, 0, label, None)
    }

    /// Looks for the next transition that matches the provided label, assuming determinism.
    ///
    /// This method is similar to {@link #step(int, int)} but is used more efficiently when
    /// iterating over multiple transitions from the same source state. It keeps the latest reached
    /// transition index in {@code transition.transitionUpto} so the next call to this method can
    /// continue from there instead of restarting from the first transition.
    ///
    /// # Arguments
    /// * `transition`: The transition to start the lookup from (inclusive, using its [Transition::source]
    ///    and [Transition::transition_upto]). It is updated with the matched transition; or with
    ///    [Transition::dest] = `None` if no match.
    /// * `label`: The codepoint to look up.
    /// 
    /// # Returns
    /// The destination state, or `None` if no matching outgoing transition.
    pub fn next(&self, t: &mut Transition, label: u32) -> Option<State> {
        self.next_state_update_transition(t.source, t.transition_upto.unwrap_or(0), label, Some(t))
    }

    /// Looks for the next transition that matches the provided label, assuming determinism.
    /// 
    /// # Arguments
    /// * `source`: The source state.
    /// * `from_transition_index`: the transition index to start the lookup from (inclusive).
    /// * `label`: The codepoint to look up.
    /// * `transition`: The transition to update with the matched transition, or `None` for no update.
    /// 
    /// # Returns
    /// The destination state, or `None` if no matching outgoing transition.
    fn next_state_update_transition(&self, source: State, from_transition_index: u32, label: u32, transition: Option<&mut Transition>) -> Option<State> {
        let mut first_transition_index = self.states[source.usize()].transitions_index.unwrap();
        let mut num_transitions = self.states[source.usize()].num_transitions;

        // Since transitions are sorted, binary search the transition for which label is within [min_label, max_label].
        let mut low = from_transition_index;
        let mut high = num_transitions - 1;

        while low <= high {
            let mid = (low + high) >> 1;
            let transition_index = (first_transition_index + mid) as usize;
            let min_label = self.transitions[transition_index as usize].min;
            if min_label > label as u32 {
                high = mid - 1
            } else {
                let max_label = self.transitions[transition_index as usize].max;
                if max_label < label as u32 {
                    low = mid + 1
                } else {
                    let dest_state = self.transitions[transition_index].to_state.unwrap();

                    if let Some(transition) = transition {
                        transition.dest = Some(dest_state);
                        transition.min = min_label;
                        transition.max = max_label;
                        transition.transition_upto = Some(mid);
                    }

                    return Some(dest_state);
                }
            }
        }

        if let Some(transition) = transition {
            transition.dest = None;
            transition.transition_upto = None;
        }

        None
    }
}

impl TransitionAccessor for Automaton {
    fn init_transition(&self, state: State, t: &mut Transition) -> usize {
        assert!(state.0 < self.get_num_states());
        t.source = state;
        t.transition_upto = self.states[state.usize()].transitions_index;
        <Self as TransitionAccessor>::get_num_transitions(self, state)
    }

    fn get_next_transition(&mut self, t: &mut Transition) {
        assert!(t.transition_upto.is_some());
        // Make sure there is still a transition left:
        assert!(t.transition_upto.unwrap() + 1 - self.states[t.source.usize()].transitions_index.unwrap() <= self.states[t.source.usize() + 1].transitions_index.unwrap());

        // Make sure transitions are in fact sorted.
        assert!(self.transition_sorted(t));

        let next = t.transition_upto.unwrap() + 1;
        t.transition_upto = Some(next);
        t.dest = Some(self.transitions[next as usize].to_state.unwrap());
        t.min = self.transitions[next as usize].min;
        t.max = self.transitions[next as usize].max;
    }

    fn get_num_transitions(&self, state: State) -> usize {
        assert!(state.0 < self.get_num_states());
        self.states[state.usize()].num_transitions as usize
    }

    fn get_transition(&self, state: State, index: usize, t: &mut Transition) {
        let i = self.states[state.usize()].transitions_index.unwrap() as usize + index;
        t.source = state;
        t.dest = Some(self.transitions[i].to_state.unwrap());
        t.min = self.transitions[i].min;
        t.max = self.transitions[i].max;
    }
}

/// Records new states and transitions and then [Builder::finish] creates the [Automaton]. Use
/// this when you cannot create the Automaton directly because it's too restrictive to have to add
/// all transitions leaving each state at once.
#[derive(Debug)]
pub struct Builder {
    next_state: u32,
    is_accept: BitVec,
    transitions: Vec<BuilderTransitionInfo>,
}

impl Builder {
    /// Build a new [Builder], pre-allocating for 16 states and transitions.
    pub fn new() -> Builder {
        Builder::with_capacity(16, 16)
    }

    /// Creates a builder with enough space for the given number of states and transitions.
    pub fn with_capacity(num_states: usize, num_transitions: usize) -> Builder {
        Builder {
            next_state: 0,
            is_accept: BitVec::with_capacity(num_states),
            transitions: Vec::with_capacity(num_transitions),
        }
    }

    /// Add a new transition with min = max = label.
    pub fn add_transition(&mut self, source: State, dest: State, label: u32) {
        self.add_transition_range(source, dest, label as u32, label as u32)
    }

    /// Add a new transition with the specified source, dest, min, max.
    pub fn add_transition_range(&mut self, source: State, dest: State, min: u32, max: u32) {
        let bti = BuilderTransitionInfo {
            source,
            dest,
            min,
            max,
        };
        
        self.transitions.push(bti);
    }

    /// Add a (virtual) epsilon transition between source and dest. Dest state must already have all
    /// transitions added because this method simply copies those same transitions over to source.
    pub fn add_epsilon(&mut self, source: State, dest: State) {
        for t in self.transitions.iter() {
            if t.dest == dest {
                self.add_transition_range(source, dest, t.min, t.max);
            }
        }

        if self.is_accept(dest) {
            self.set_accept(source, true);
        }
    }

    /// Compiles all added states and transitions into a new {@code Automaton} and returns it.
    pub fn finish(self) -> Automaton {
        // Create automaton with the correct size.
        let num_states = self.next_state as usize;
        let num_transitions = self.transitions.len();

        let mut a = Automaton::new(num_states, num_transitions);

        // Create all states
        for state in 0..num_states {
            let a_state = a.create_state();
            assert_eq!(a_state.usize(), state);
            a.set_accept(a_state, self.is_accept(a_state));
        }

        // Create all transitions
        self.transitions.sort();
        for t in self.transitions {
            a.add_transition_range(t.source, t.dest, t.min, t.max);
        }
        
        a.finish_state();

        a
    }

    /// Create a new state.
    pub fn create_state(&mut self) -> State {
        let state = State(self.next_state);
        self.next_state += 1;
        state
    }

    /// Returns true if the specified state is an accept state.
    pub fn is_accept(&self, state: State) -> bool {
        self.is_accept[state.usize()]
    }

    /// Set or clear this state as an accept state.
    pub fn set_accept(&mut self, state: State, accept: bool) {
        self.is_accept.set(state.usize(), accept);
    }

    /// Returns the number of states in this automaton.
    pub fn get_num_states(&self) -> u32 {
        self.next_state
    }

    /// Copies over all states/transitions from other.
    pub fn copy(&mut self, other: &Automaton) {
        let offset = self.get_num_states();
        let other_num_states = other.get_num_states();

        // Copy all states.
        self.copy_states(other);

        // Copy all transitions.
        let mut t = Transition::default();
        for state in 0..other_num_states {
            let count = other.init_transition(State(state), &mut t);
            for i in 0..count {
                other.get_next_transition(&mut t);
                let dest = t.dest.unwrap();
                self.add_transition_range(
                    State(offset + state), State(offset + dest.0), t.min, t.max);
            }
        }
    }

    // Copies over all states from other.
    pub fn copy_states(&mut self, other: &Automaton) {
        let other_num_states = other.get_num_states();
        for s in 0..other_num_states {
            let new_state = self.create_state();
            self.set_accept(new_state, other.is_accept(State(s)));
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuilderTransitionInfo {
    source: State,
    dest: State,
    min: u32,
    max: u32,
}

impl Default for BuilderTransitionInfo {
    fn default() -> Self {
        BuilderTransitionInfo {
            source: State(0),
            dest: State(0),
            min: 0,
            max: 0,
        }
    }
}

/// Sort by source, min, max, and dest, respectively.
impl PartialOrd for BuilderTransitionInfo {
    fn partial_cmp(&self, other: &BuilderTransitionInfo) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BuilderTransitionInfo {
    fn cmp(&self, other: &BuilderTransitionInfo) -> Ordering {
        match self.source.cmp(&other.source) {
            Ordering::Equal => match self.min.cmp(&other.min) {
                Ordering::Equal => match self.max.cmp(&other.max) {
                    Ordering::Equal => self.dest.cmp(&other.dest),
                    o => o,
                },
                o => o,
            },
            o => o,
        }
    }
}
