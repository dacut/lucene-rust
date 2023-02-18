use {
    crate::util::automaton::{
        compiled_automaton::{CompiledAutomaton, AutomatonType},
        byte_runnable::ByteRunnable, transition::Transition, transition_accessor::TransitionAccessor,
    },
    std::fmt::Debug,
};

/// A FilteredTermsEnum that enumerates terms based upon what is accepted by a DFA.
///
/// The algorithm is such:
///
/// * As long as matches are successful, keep reading sequentially.
/// * When a match fails, skip to the next string in lexicographic order that does not enter a
///   reject state.
///
/// The algorithm does not attempt to actually skip to the next string that is completely
/// accepted. This is not possible when the language accepted by the FSM is not finite (i.e. *
/// operator).
pub struct AutomatonTermsEnum<'a, FTE> {
    // filtered terms enum we are wrapping.
    filtered_terms_enum: FTE,

    // a tableized array-based form of the DFA
    byte_runnable: Box<&'a dyn ByteRunnable>,

    // common suffix of the automaton
    common_suffix_ref: Option<&'a [u8]>,

    // true if the automaton accepts a finite language
    finite: bool,

    // array of sorted transitions for each state, indexed by state number
    transition_accessor: Box<&'a dyn TransitionAccessor>,

    // Used for visited state tracking: each short records gen when we last
    // visited the state; we use gens to avoid having to clear
    visited: Option<Vec<u16>>,

    cur_gen: u16,

    // the reference used for seeking forwards through the term dictionary
    seek_bytes_ref: Vec<u8>,

    // true if we are enumerating an infinite portion of the DFA.
    // in this case it is faster to drive the query based on the terms dictionary.
    // when this is true, linearUpperBound indicate the end of range
    // of terms where we should simply do sequential reads instead.
    linear: bool,

    linear_upper_bound: Vec<u8>,

    transition: Transition,

    saved_states: Vec<u16>,
}

impl<'a, FTE> AutomatonTermsEnum<'a, FTE> {
    pub fn new(fte: FTE, compiled: &'a CompiledAutomaton) -> Self {
        match compiled {
            CompiledAutomaton::Normal(compiled) => {
                let finite = compiled.is_finite();
                let byte_runnable = compiled.get_byte_runnable().unwrap();
        
                Self {
                    filtered_terms_enum: fte,
                    byte_runnable,
                    common_suffix_ref: compiled.get_common_suffix_ref(),
                    finite: compiled.is_finite(),
                    transition_accessor: compiled.get_transition_accessor().unwrap(),
                    visited: if finite { None } else { Some(vec![0; byte_runnable.size()]) },
                    cur_gen: 0,
                    seek_bytes_ref: Vec::new(),
                    linear: false,
                    linear_upper_bound: Vec::new(),
                    transition: Transition::default(),
                    saved_states: Vec::new(),
                }                        
            }
            _ => panic!("use CompiledAutomaton::get_terms_enum instead"),
        }
    }

    /// Records
}

// TODO: implement FilteredTermsEnum
