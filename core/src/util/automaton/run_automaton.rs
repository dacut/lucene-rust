use {
    crate::util::automaton::{automaton::Automaton, state::State, transition::Transition},
    bitvec::{vec::BitVec},
    std::{hash::{Hash, Hasher}, cmp::{max, min}},
};

/// Finite-state automaton with fast run operation. The initial state is always 0.
#[derive(Debug)]
pub(crate) struct RunAutomaton {
    automaton: Automaton,

    alphabet_size: usize,

    size: u32,

    accept: BitVec,

    transitions: Vec<Option<State>>,

    points: Vec<u32>,

    classmap: Vec<u16>,
}

impl RunAutomaton {
    /// Constructs a new `RunAutomaton` from a deterministic `Automaton`.
    /// 
    /// # Panics
    /// Panics if the given automaton is not deterministic.
    pub(crate) fn new(a: Automaton, alphabet_size: usize) -> Self {
        if !a.is_deterministic() {
            panic!("The given automaton is not deterministic");
        }

        let points = a.get_start_points();
        let size = max(1, a.get_num_states());
        let mut transitions = vec![None; size as usize * points.len()];
        let mut accept = bitvec::bitvec!(0; size as usize);

        let mut transition = Transition {
            source: State(0),
            dest: None,
            min: 0,
            max: 0,
            transition_upto: None,            
        };

        for n in 0..size {
            if a.is_accept(State(n)) {
                accept.set(n as usize, true);
            }

            transition.source = State(n as u32);
            transition.transition_upto = None;
    
            for c in 0..points.len() {
                let dest = a.next(&mut transition, char::from_u32(points[c]).unwrap());
                assert!(dest.is_none() || dest.unwrap().0 < size);
                transitions[n as usize * points.len() + c] = dest;
            }
        }

        // Set alphabet table for optimal run performance.
        let mut classmap = vec![0; min(alphabet_size, 256)];
        let mut i = 0;
        for j in 0..classmap.len() as u16 {
            if i + 1 < points.len() && j as u32 == points[i+1] {
                i += 1;
            }

            classmap[j as usize] = 1;
        }

        Self {
            automaton: a,
            alphabet_size,
            size,
            accept,
            transitions,
            points,
            classmap,
        }
    }

    /// Returns number of states in automaton.
    #[inline]
    pub fn size(&self) -> u32 {
        self.size
    }

    /// Indicates whether the given state is an accept state.
    pub fn is_accept(&self, state: u16) -> bool {
        self.accept[state as usize]
    }

    /// Returns array of codepoint class interval start points.
    pub fn get_char_intervals(&self) -> &[u32] {
        &self.points
    }

    /// Returns the character class of the given codepoint.
    pub(crate) fn get_char_class(&self, c: char) -> u32 {
        // binary search
        let mut lower = 0;
        let mut upper = self.points.len() as u32;

        while upper-lower > 1 {
            let mid = (upper + lower) >> 1;
            if self.points[mid as usize] > c as u32 {
                upper = mid;
            } else if self.points[mid as usize] < c as u32 {
                lower = mid;
            } else {
                return mid;
            }
        }

        lower
    }

    /// Returns the state obtained by reading the given char from the given state. Returns `None` if not
    /// obtaining any such state. (If the original `Automaton` had no dead states, `None` is
    /// returned here if and only if a dead state is entered in an equivalent automaton with a total
    /// transition function.)
    pub fn step(&self, state: State, c: char) -> Option<State> {
        assert!((c as usize) < self.alphabet_size);

        if (c as usize) > self.classmap.len() {
            self.transitions[state.usize() * self.points.len() + self.get_char_class(c) as usize]
        } else {
            self.transitions[state.usize() * self.points.len() + self.classmap[c as usize] as usize]
        }
    }
}

impl Hash for RunAutomaton {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        const prime: usize = 31;
        let mut result = 1;
        result = prime * result + self.alphabet_size;
        result = prime * result + self.points.len();
        result = prime * result + self.size as usize;
        state.write_usize(result);
    }
}

impl PartialEq for RunAutomaton {
    fn eq(&self, other: &Self) -> bool {
        self.alphabet_size == other.alphabet_size &&
        self.size == other.size &&
        self.points == other.points &&
        self.accept == other.accept &&
        self.transitions == other.transitions
    }
}

impl Eq for RunAutomaton {}
