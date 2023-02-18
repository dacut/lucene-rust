use {
    crate::{
        util::automaton::{
            automaton::Automaton,
            byte_runnable::ByteRunnable,
            byte_run_automaton::ByteRunAutomaton,
            nfa_run_automaton::NFARunAutomaton,
            operations,
            transition_accessor::TransitionAccessor,
        },
    }
};

/// Automata are compiled into different internal forms for the most efficient execution depending
/// upon the language they accept.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutomatonType {
    /// Automaton that accepts no strings.
    None,

    /// Automaton that accepts all possible strings.
    All,

    /// Automaton that accepts only a single fixed string.
    Single,

    /// Catch-all for any other automata.
    Normal,
}

/// Immutable class holding compiled details for a given Automaton. The Automaton could either be
/// deterministic or non-deterministic, For deterministic automaton, it must not have dead states but
/// is not necessarily minimal. And will be executed using [ByteRunAutomaton]. For
/// non-deterministic automaton, it will be executed using [NFARunAutomaton].
/// 
/// # Rust notes
/// This class has been Rustified into an enum that dispatches onto the different types of automata.
#[derive(Debug)]
pub enum CompiledAutomaton {
    /// Automaton that accepts no strings.
    None,

    /// Automaton that accepts all possible strings.
    All,

    /// Automaton that accepts only a single fixed string.
    Single(CompiledAutomatonSingle),

    /// Catch-all for any other automata.
    Normal(CompiledAutomatonNormal),
}

impl CompiledAutomaton {
    /// Create a new CompiledAutomaton, passing `simplify = true` so we try to simplify the automaton
    pub fn new(automaton: Automaton) -> Self {
        Self::new_with_simplification(automaton, false, true)
    }


    /// Create a new CompiledAutomaton. If simplify is true, we run possibly expensive operations to determine if the
    /// automaton is one of the special cases. Set finite to true if
    /// the automaton is finite, otherwise set to false if infinite or you don't know.
    pub fn new_with_simplification(automaton: Automaton, finite: bool, simplify: bool) -> Self {
        Self::new_with_simplification_and_binary(automaton, finite, simplify, false)
    }

    /// Create a new CompiledAutomaton. If simplify is true, we run possibly expensive operations to determine if the
    /// automaton is one of the special cases. Set finite to true if
    /// the automaton is finite, otherwise set to false if infinite or you don't know.
    pub fn new_with_simplification_and_binary(mut automaton: Automaton, finite: bool, simplify: bool, is_binary: bool) -> Self {
        if automaton.get_num_states() == 0 {
            automaton = Automaton::default();
            automaton.create_state();
        }

        // simplify requires a DFA
        if simplify && automaton.is_deterministic() {
            // Test whether the automaton is a "simple" form and
            // if so, don't create a runAutomaton.  Note that on a
            // large automaton these tests could be costly:

            if operations::is_empty(automaton) {
                // matches nothing
                return Self::None
            }

            // NOTE: only approximate, because automaton may not be minimal:
            let is_total = if is_binary {
                operations::is_total_range(automaton, 0, 0xff)
            } else {
                operations::is_total(automaton)
            };

            if is_total {
                // matches all possible strings
                return Self::All
            }
        
            let singleton: Option<&[u32]> = operations::get_singleton(automaton);

            if let Some(singleton) = singleton {
                // matches a fixed string
                
                let term = if is_binary {
                    StringHelper::ints_ref_to_bytes_ref(singleton)
                } else {
                    BytesRef::new(
                        UnicodeUtil::new_string(singleton.ints, singleton.offset, singleton.length))
                };

                return Self::Single(CompiledAutomatonSingle {
                    term,
                    automaton,
                })
            }
        }
    

        //type = AUTOMATON_TYPE.NORMAL;
        //term = null;
        //this.finite = finite;

        let binary: Automaton = if is_binary {
            // Caller already built binary automaton themselves, e.g. PrefixQuery
            // does this since it can be provided with a binary (not necessarily
            // UTF8!) term:
            automaton
        } else {
            // Incoming automaton is unicode, and we must convert to UTF8 to match what's in the index:
            UTF32ToUTF8::convert(automaton);
        };

        // compute a common suffix for infinite DFAs, this is an optimization for "leading wildcard"
        // so don't burn cycles on it if the DFA is finite, or largeish
        let common_suffix_ref = if finite || automaton.get_num_states() + automaton.get_num_transitions() > 1000 {
            None
        } else {
            let suffix = operations::get_common_suffix_bytes_ref(binary);
            if suffix.length == 0 {
                None
            } else {
                Some(suffix)
            }
        };

        if !automaton.is_deterministic() && !binary.is_deterministic() {
            Self::Normal(CompiledAutomatonNormal {
                automaton: None,
                run_automaton: None,
                sink_state: None,
                nfa_run_automaton: Some(NFARunAutomaton::new(binary, 0xff)),
            })
        } else {
            // We already had a DFA (or threw exception), according to mike UTF32toUTF8 won't "blow up"
            let binary = operations::determinize(binary, i32::MAX);
            let run_automaton = ByteRunAutomaton::new(binary, true);
            // TODO: this is a bit fragile because if the automaton is not minimized there could be more
            // than 1 sink state but auto-prefix will fail
            // to run for those:
            let sink_state = Self::find_sink_state(run_automaton.automaton);

            Self::Normal(CompiledAutomatonNormal {
                automaton: Some(run_automaton.automaton),
                run_automaton: Some(run_automaton),
                sink_state: Some(sink_state),
                nfa_run_automaton: None,
            })
        }
    }

    /// Returns the type of Automaton.
    pub fn get_type(&self) -> AutomatonType {
        match self {
            Self::None => AutomatonType::None,
            Self::All => AutomatonType::All,
            Self::Single(_) => AutomatonType::Single,
            Self::Normal(_) => AutomatonType::Normal,
        }
    }
}

/// CompiledAutomaton for [AutomatonType::Normal].
#[derive(Debug)]
pub struct CompiledAutomatonSingle {
    /// The singleton term.
    term: Vec<u8>,
}

impl CompiledAutomatonSingle {
    /// Return the term for the automaton.
    pub fn get_term(&self) -> &[u8] {
        &self.term
    }
}

/// CompiledAutomaton for [AutomatonType::Normal].
#[derive(Debug)]
pub struct CompiledAutomatonNormal {
    /// Matcher for quickly determining if a byte[] is accepted.
    run_automaton: Option<ByteRunAutomaton>,

    /// Two dimensional array of transitions, indexed by state number for traversal. The state
    /// numbering is consistent with [::CompiledAutomaton::run_automaton].
    automaton: Option<Automaton>,

    /// Matcher directly run on a NFA, it will determinize the state on need and caches it, note that
    /// this field and [::run_automaton] will not be `None` at the same time
    ///
    /// TODO: merge this with run_automaton
    nfa_run_automaton: Option<NFARunAutomaton>,

    /// Shared common suffix accepted by the automaton. Only valid when the automaton accepts an
    /// infinite language. This will be `None` if the common prefix is length 0.
    common_suffix_ref: Option<Vec<u8>>,

    /// Indicates if the automaton accepts a finite set of strings.
    finite: bool,

    /// Which state, if any, accepts all suffixes, else `None`.
    sink_state: Option<usize>,
}

impl CompiledAutomatonNormal {
    pub fn get_run_automaton(&self) -> Option<&ByteRunAutomaton> {
        self.run_automaton.as_ref()
    }

    pub fn get_automaton(&self) -> Option<&Automaton> {
        self.automaton.as_ref()
    }

    pub fn get_nfa_run_automaton(&self) -> Option<&NFARunAutomaton> {
        self.nfa_run_automaton.as_ref()
    }

    pub fn get_common_suffix_ref(&self) -> Option<&[u8]> {
        self.common_suffix_ref.as_ref().map(|v| v.as_slice())
    }

    pub fn is_finite(&self) -> bool {
        self.finite
    }

    pub fn get_sink_state(&self) -> Option<usize> {
        self.sink_state
    }

    /// Get a [ByteRunnable] instance, it will be different depending on whether a NFA or DFA is
    /// passed in.
    pub fn get_byte_runnable(&self) -> Option<Box<&dyn ByteRunnable>> {
        // they can be both null but not both non-null
        assert!(self.nfa_run_automaton.is_none() || self.run_automaton.is_none());

        if let Some(nfa_run_automaton) = &self.nfa_run_automaton {
            Some(Box::new(nfa_run_automaton))
        } else if let Some(run_automaton) = &self.run_automaton {
            Some(Box::new(run_automaton))
        } else {
            None
        }
    }

    /// Get a [TransitionAccessor] instance, it will be different depending on whether a NFA or DFA
    /// is passed in
    pub fn get_transition_accessor(&self) -> Option<Box<&dyn TransitionAccessor>> {
        assert!(self.nfa_run_automaton.is_none() || self.automaton.is_none());

        if let Some(nfa_run_automaton) = &self.nfa_run_automaton {
            Some(Box::new(nfa_run_automaton))
        } else if let Some(automaton) = &self.automaton {
            Some(Box::new(automaton))
        } else {
            None
        }
    }
}
