use {
    crate::util::automaton::{
        automata,
        automaton::{Automaton, Builder},
        frozen_int_set::FrozenIntSet,
        int_set::IntSet,
        state::State,
        state_set::StateSet,
        transition::Transition,
        transition_accessor::TransitionAccessor,
        too_complex_to_determinize_error::TooComplexToDeterminizeError,
    },
    crate::util::hppc::bit_mixer::mix32_u32,
    std::{
        cmp::{Ordering, PartialOrd, Ord},
        collections::{HashMap, HashSet, VecDeque},
        hash::{Hash, Hasher},
        mem::swap,
    },
    bitvec::prelude::*,
};

/// Default maximum effort that [Operations::determinize] should spend before giving up and
/// throwing [TooComplexToDeterminizeException].
pub const DEFAULT_DETERMINIZE_WORK_LIMIT: usize = 10000;

/// Maximum level of recursion allowed in recursive operations.
pub const MAX_RECURSION_LEVEL: usize = 1000;

/// Automata operations.
/// Returns an automaton that accepts the concatenation of the languages of the given automata.
///
/// Complexity: linear in total number of states.
pub fn concatenate(a1: Automaton, a2: Automaton) -> Automaton {
    concatenate_many(&vec![&a1, &a2])
}

/// Returns an automaton that accepts the concatenation of the languages of the given automata.
///
/// Complexity: linear in total number of states.
pub fn concatenate_many(l: &[&Automaton]) -> Automaton {
    let mut result = Automaton::default();

    // First pass: create all states
    for a in l.iter() {
        let num_states = a.get_num_states();

        if num_states == 0 {
            result.finish_state();
            return result;
        }

        for s in 0..num_states {
            result.create_state();
        }
    }

    // Second pass: add transitions, carefully linking accept
    // states of A to init state of next A:
    let mut state_offset: u32 = 0;
    let mut t = Transition::default();

    for (i, a) in l.iter().enumerate() {
        let num_states = a.get_num_states();

        let next_a = l.get(i + 1);
        for s in 0..num_states {
            let num_transitions = a.init_transition(State(s as u32), &mut t);

            for j in 0..num_transitions {
                a.get_next_transition(&mut t);
                let dest = t.dest.unwrap();
                result.add_transition_range(State(state_offset + s as u32), State(state_offset + dest.0), t.min, t.max);
            }

            if a.is_accept(State(s)) {
                let follow_a = next_a;
                let mut follow_offset = state_offset;

                let mut upto = i + 1;
                loop {
                    if let Some(follow_a) = follow_a {
                        // Adds a "virtual" epsilon transition:
                        let num_transitions = follow_a.init_transition(State(0), &mut t);
                        for j in 0..num_transitions {
                            follow_a.get_next_transition(&mut t);
                            let dest = t.dest.unwrap();
                            result.add_transition_range(
                                State(state_offset + s),
                                State(follow_offset + num_states + dest.0),
                                t.min,
                                t.max,
                            );
                        }
                        if follow_a.is_accept(State(0)) {
                            // Keep chaining if followA accepts empty string
                            follow_offset += follow_a.get_num_states();
                            follow_a = l.get(upto + 1).unwrap();
                            upto += 1;
                        } else {
                            break;
                        }
                    } else {
                        result.set_accept(State(state_offset + s), true);
                        break;
                    }
                }
            }
        }

        state_offset += num_states;
    }

    if result.get_num_states() == 0 {
        result.create_state();
    }

    result.finish_state();

    result
}

/// Returns an automaton that accepts the union of the empty string and the language of the given
/// automaton. This may create a dead state.
///
/// Complexity: linear in number of states.
pub fn optional(a: &Automaton) -> Automaton {
    let mut result = Automaton::default();
    result.create_state();
    result.set_accept(State(0), true);

    if a.get_num_states() > 0 {
        result.copy(&a);
        result.add_epsilon(State(0), State(1));
    }

    result.finish_state();
    result
}

/// Returns an automaton that accepts the Kleene star (zero or more concatenated repetitions) of
/// the language of the given automaton. Never modifies the input automaton language.
///
/// Complexity: linear in number of states.
pub fn repeat(a: &Automaton) -> Automaton {
    if a.get_num_states() == 0 {
        // Repeating the empty automata will still only accept the empty automata.
        return a.clone();
    }

    let mut builder = Builder::new();
    builder.create_state();
    builder.set_accept(State(0), true);
    builder.copy(a);

    let mut t = Transition::default();
    let count = a.init_transition(State(0), &mut t);
    for i in 0..count {
        a.get_next_transition(&mut t);
        let dest = t.dest.unwrap();
        builder.add_transition_range(State(0), State(dest.0 + 1), t.min, t.max);
    }

    let num_states = a.get_num_states();
    for s in 0..num_states {
        if a.is_accept(State(s)) {
            let count = a.init_transition(State(0), &mut t);
            for i in 0..count {
                a.get_next_transition(&mut t);
                let dest = t.dest.unwrap();
                builder.add_transition_range(State(s + 1), State(dest.0 + 1), t.min, t.max);
            }
        }
    }

    builder.finish()
}

/// Returns an automaton that accepts `min` or more concatenated repetitions of the
/// language of the given automaton.
///
/// Complexity: linear in number of states and in `min`.
pub fn repeat_n(a: &Automaton, count: usize) -> Automaton {
    let kleene = repeat(a);

    if count == 0 {
        return kleene;
    }

    let mut a_list = Vec::with_capacity(count + 1);
    for _ in 0..count {
        a_list.push(a);
    }

    a_list.push(&kleene);
    concatenate_many(&a_list)
}

/// Returns an automaton that accepts between `min` and `max` (including
/// both) concatenated repetitions of the language of the given automaton.
///
/// Complexity: linear in number of states and in `min` and `max`.
pub fn repeat_range(a: &Automaton, min: usize, max: usize) -> Automaton {
    if min > max {
        return automata::make_empty();
    }

    let mut b = match min {
        0 => automata::make_empty_string(),
        1 => {
            let mut b = Automaton::default();
            b.copy(a);
            b
        }
        _ => {
            let mut a_list = vec![a; min];
            concatenate_many(&a_list)
        }
    };

    let mut prev_accept_states = to_set(&b, 0);
    let builder = Builder::new();
    builder.copy(&b);

    for i in min..max {
        let num_states = builder.get_num_states();
        builder.copy(a);
        for s in prev_accept_states {
            builder.add_epsilon(State(s), State(num_states));
        }

        prev_accept_states = to_set(a, num_states);
    }

    builder.finish()
}

fn to_set(a: &Automaton, offset: u32) -> HashSet<u32> {
    let num_states = a.get_num_states();
    let is_accept = a.get_accept_states();
    let result = HashSet::new();
    for state in is_accept.iter_ones() {
        result.insert(offset + state as u32);
    }

    result
}

/// Returns a (deterministic) automaton that accepts the complement of the language of the given
/// automaton.
///
/// Complexity: linear in number of states if already deterministic and exponential otherwise.
///
/// # Parameters
/// * `determinize_work_limit`: maximum effort to spend determinizing the automaton. Set higher to
/// allow more complex queries and lower to prevent memory exhaustion. [DEFAULT_DETERMINIZE_WORK_LIMIT]
///  is a good starting default.
pub fn complement(a: &Automaton, determinize_work_limit: usize) -> Result<Automaton, TooComplexToDeterminizeError> {
    let mut a = totalize(determinize(a, determinize_work_limit)?);
    let num_states = a.get_num_states();
    for p in 0..num_states {
        a.set_accept(State(p), !a.is_accept(State(p)));
    }

    return Ok(remove_dead_states(a));
}

/// Returns a (deterministic) automaton that accepts the intersection of the language of `a1`
/// and the complement of the language of `a2`. As a side-effect, the automata
/// may be determinized, if not already deterministic.
///
/// Complexity: quadratic in number of states if `a2` already deterministic and exponential in
/// number of `a2`'s states otherwise.
///
/// # Parameters
/// * `a1`: the initial automaton
/// * `a2`: the automaton to subtract
/// * `determinize_work_limit`: maximum effort to spend determinizing the automaton. Set higher to
/// allow more complex queries and lower to prevent memory exhaustion. [DEFAULT_DETERMINIZE_WORK_LIMIT]
///  is a good starting default.
pub fn minus(a1: &Automaton, a2: &Automaton, determinize_work_limit: usize) -> Result<Automaton, TooComplexToDeterminizeError> {
    if is_empty(a1) {
        Ok(automata::make_empty())
    } else if is_empty(a2) {
        Ok(a1.clone())
    } else {
        Ok(intersection(a1, &complement(a2, determinize_work_limit)?))
    }
}

/// Returns an automaton that accepts the intersection of the languages of the given automata.
/// Never modifies the input automata languages.
///
/// Complexity: quadratic in number of states.
pub fn intersection(a1: &Automaton, a2: &Automaton) -> Automaton {
    todo!()
}

/// Returns true if these two automata accept exactly the same language. This is a costly
/// computation! Both automata must be determinized and have no dead states!
pub fn same_language(a1: &Automaton, a2: &Automaton) -> bool {
    subset_of(a2, a1) && subset_of(a1, a2)
}

/// Returns true if this automaton has any states that cannot be reached from the initial state or
/// cannot reach an accept state. Cost is O(numTransitions+numStates).
pub fn has_dead_states(a: &Automaton) -> bool {
    let live_states = get_live_states(a);
    let num_live = live_states.count_ones();
    let num_states = a.get_num_states() as usize;
    assert!(num_live <= num_states, "num_live={num_live}, num_states={num_states}, {live_states:?}");
    return num_live < num_states;
}

/// Returns true if there are dead states reachable from an initial state.
pub fn has_dead_states_from_initial(a: &Automaton) -> bool {
    let reachable_from_initial = get_live_states_from_initial(a);
    let reachable_from_accept = get_live_states_to_accept(a);

    let dead_from_initial = reachable_from_initial & !reachable_from_accept;
    dead_from_initial.count_ones() > 0
}

/// Returns true if there are dead states that reach an accept state.
pub fn has_dead_states_to_accept(a: &Automaton) -> bool {
    let reachable_from_initial = get_live_states_from_initial(a);
    let reachable_from_accept = get_live_states_to_accept(a);

    let dead_to_accept = reachable_from_accept & !reachable_from_initial;
    dead_to_accept.count_ones() > 0
}

/// Returns true if the language of `a1` is a subset of the language of `a2`.
/// Both automata must be determinized and must have no dead states.
///
/// Complexity: quadratic in number of states.
pub fn subset_of(a1: &Automaton, a2: &Automaton) -> bool {
    assert!(a1.is_deterministic(), "a1 must be deterministic");
    assert!(a2.is_deterministic(), "a2 must be deterministic");
    assert!(!has_dead_states_from_initial(a1), "a1 must have no dead states");
    assert!(!has_dead_states_from_initial(a2), "a2 must have no dead states");

    if a1.get_num_states() == 0 {
        // Empty language is always a subset of any other language.
        return true;
    }

    if a2.get_num_states() == 0 {
        return is_empty(a1);
    }

    // TODO: cutover to iterators instead
    let transitions1 = a1.get_sorted_transitions();
    let transitions2 = a2.get_sorted_transitions();
    let mut worklist = VecDeque::new();
    let mut visited = HashSet::new();
    let p = (State(0), State(0));
    worklist.push_back(p);
    visited.insert(p);

    while !worklist.is_empty() {
        let p = worklist.pop_front().unwrap();
        if a1.is_accept(p.0) && !a2.is_accept(p.1) {
            return false
        }

        let t1 = transitions1.as_slice()[p.0.usize()];
        let t2 = transitions2.as_slice()[p.1.usize()];

        let mut b2 = 0;
        for n1 in 0..t1.len() {
            while b2 < t2.len() && t2[b2].max < t1[n1].min {
                b2 += 1;
            }

            let min1 = t1[n1].min;
            let max1 = t1[n1].max;

            for n2 in b2..t2.len() {
                if t1[n1].max < t2[n2].min {
                    break;
                }
                if t2[n2].min > min1 {
                    return false;
                }
                if t2[n2].max < 0 {
                    min1 = t2[n2].max + 1;
                } else {
                    min1 = char::MAX as u32;
                    max1 = 0;
                }

                let q = (t1[n1].dest.unwrap(), t2[n2].dest.unwrap());
                if !visited.contains(&q) {
                    worklist.push_back(q);
                    visited.insert(q);
                }
            }

            if min1 <= max1 {
                return false;
            }
        }
    }

    true
}

/// Returns an automaton that accepts the union of the languages of the given automata.
///
/// Complexity: linear in number of states.
pub fn union(a1: &Automaton, a2: &Automaton) -> Automaton {
    union_many(&vec![a1, a2])
}

/// Returns an automaton that accepts the union of the languages of the given automata.
///
/// Complexity: linear in number of states.
pub fn union_many(l: &[&Automaton]) -> Automaton {
    let mut result = Automaton::default();

    // Create initial state:
    result.create_state();

    // Copy over all automata
    for a in l {
        result.copy(a);
    }

    // Add epsilon transition from new initial state;
    let mut state_offset = 1;
    for a in l {
        if a.get_num_states() == 0 {
            continue;
        }

        result.add_epsilon(State(0), State(state_offset));
        state_offset += a.get_num_states();
    }

    result.finish_state();
    remove_dead_states(&result)
}

struct TransitionInfo {
    dest: State,
    min: u32,
    max: u32,
}

// Holds all transitions that start on this int point, or end at this point-1.
struct PointTransitions {
    point: u32,
    ends: Vec<TransitionInfo>,
    starts: Vec<TransitionInfo>,
}

impl PointTransitions {
    fn new(point: u32) -> Self {
        PointTransitions {
            point,
            ends: Vec::new(),
            starts: Vec::new(),
        }
    }

    fn reset(&mut self, point: u32) {
        self.point = point;
        self.ends.clear();
        self.starts.clear();
    }

    fn add_starts(&mut self, t: &Transition) {
        self.starts.push(TransitionInfo {
            dest: t.dest.unwrap(),
            min: t.min,
            max: t.max,
        });
    }

    fn add_ends(&mut self, t: &Transition) {
        self.ends.push(TransitionInfo {
            dest: t.dest.unwrap(),
            min: t.min,
            max: t.max,
        });
    }
}

impl Eq for PointTransitions {}

impl Ord for PointTransitions {
    fn cmp(&self, other: &Self) -> Ordering {
        self.point.cmp(&other.point)
    }
}

impl Hash for PointTransitions {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.point.hash(state);
    }
}

impl PartialEq for PointTransitions {
    fn eq(&self, other: &Self) -> bool {
        self.point == other.point
    }
}

impl PartialOrd for PointTransitions {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.point.partial_cmp(&other.point)
    }
}

struct PointTransitionSet {
    points: Vec<PointTransitions>,
}

impl PointTransitionSet {
    fn new() -> Self {
        PointTransitionSet {
            points: Vec::with_capacity(5),
        }
    }

    fn len(&self) -> usize {
        self.points.len()
    }

    fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    fn next(&mut self, point: u32) -> &mut PointTransitions {
        let pt = PointTransitions::new(point);
        let pos = self.points.len();
        self.points.push(pt);
        &mut self.points.as_mut_slice()[pos]
    }

    fn find(&self, point: u32) -> Option<&PointTransitions> {
        self.points.iter().find(|pt| pt.point == point)
    }

    fn reset(&mut self) {
        self.points.clear();
    }

    fn sort(&mut self) {
        self.points.sort();
    }

    fn add(&mut self, t: &Transition) {
        let min_pt = self.find(t.min).unwrap();
        let max_pt = self.find(t.max).unwrap();
        min_pt.add_starts(t);
        max_pt.add_ends(t);
    }
}

/// Determinizes the given automaton.
///
/// Worst case complexity: exponential in number of states.
///
/// # Parameters
/// * `work_limit` Maximum amount of "work" that the powerset construction will spend before
///   throwing [TooComplexToDeterminizeError]. Higher numbers allow this operation to
///   consume more memory and CPU but allow more complex automatons. Use [DEFAULT_DETERMINIZE_WORK_LIMIT]
///   as a decent default if you don't otherwise know what to specify.
/// 
/// # Errors
/// TooComplexToDeterminizeError if determinizing requires more than `work_limit` "effort".
pub fn determinize(a: &Automaton, work_limit: usize) -> Result<Automaton, TooComplexToDeterminizeError> {
    if a.is_deterministic() {
        // Already determinized
        return Ok(a.clone());
    }

    if a.get_num_states() <= 1 {
        // Already determinized
        return Ok(a.clone());
    }

    // subset construction
    let mut b = Builder::new();

    // Same initial values and state will always have the same hashCode
    let initial_set = FrozenIntSet::new(vec![0], 0, State(mix32_u32(0) + 1));

    // Create state 0:
    b.create_state();

    let mut worklist = VecDeque::new();
    let newstate: HashMap<IntSet, State> = HashMap::new();
    worklist.push_back(initial_set);

    b.set_accept(State(0), a.is_accept(State(0)));
    newstate.insert(initial_set.into(), State(0));

    // like Set<Integer,PointTransitions> ???
    let points = PointTransitionSet::new();

    // like HashMap<Integer,Integer>, maps state to its count
    let states_set = StateSet::with_capacity(5);

    let mut t = Transition::default();
    let mut effort_spent = 0;
    
    // LUCENE-9981: approximate conversion from what used to be a limit on number of states, to
    // maximum "effort":
    let effort_limit = work_limit * 10;

    while !worklist.is_empty() {
        // TODO (LUCENE-9983): these int sets really do not need to be sorted, and we are paying
        // a high (unecessary) price for that!  really we just need a low-overhead Map<int,int>
        // that implements equals/hash based only on the keys (ignores the values).  fixing this
        // might be a bigspeedup for determinizing complex automata
        let s = worklist.pop_front().unwrap();

        // LUCENE-9981: we more carefully aggregate the net work this automaton is costing us, instead
        // of (overly simplistically) counting number
        // of determinized states:
        effort_spent += s.len();

        if effort_spent >= effort_limit {
            return Err(TooComplexToDeterminizeError::new(work_limit));
        }

        // Collate all outgoing transitions by min/1+max:
        for s0 in s.values() {
            let s0 = State(*s0);
            let num_transitions = TransitionAccessor::get_num_transitions(a, s0);
            a.init_transition(s0, &mut t);
            for j in 0..num_transitions {
                a.get_next_transition(&mut t);
                points.add(&t);
            }
        }

        if points.is_empty() {
            // No outgoing transitions -- skip it
            continue;
        }

        points.sort();

        let mut last_point = None;
        let mut acc_count = 0;
        let r = s.get_state();
        
        for i in 0..points.len() {
            let point = points.points[i].point;

            if !states_set.is_empty() {
                assert!(last_point.is_some());
          
                let mut q = newstate.get(&states_set.into());
                let q = match q {
                    None => {
                        let q = b.create_state();
                        let p = states_set.freeze(q);
            
                        worklist.push_back(p);
                        b.set_accept(q, acc_count > 0);
                        newstate.insert(p.into(), q);
                        q
                    }
                    Some(q) => {
                        assert!((acc_count > 0) == b.is_accept(*q),
                        "acc_count={acc_count} vs existing accept={} states={states_set:?}", b.is_accept(*q));
                        *q
                    }
                };

                b.add_transition_range(r, q, last_point.unwrap(), point - 1);
            }

            // process transitions that end on this point
            // (closes an overlapping interval)
            let transitions = points.points[i].ends;
            for j in 0..transitions.len() {
                let dest = transitions[j].dest;
                states_set.decr(dest);
                if a.is_accept(dest) {
                    acc_count -= 1;
                }
            }

            points.points[i].ends.clear();

            // process transitions that start on this point
            // (opens a new interval)
            let transitions = points.points[i].starts;
            for j in 0..transitions.len() {
                let dest = transitions[j].dest;
                states_set.incr(dest);
                if a.is_accept(dest) {
                    acc_count += 1;
                }
            }

            last_point = Some(point);
            points.points[i].starts.clear();
        }

        points.reset();
        assert!(states_set.is_empty(), "states_set is not empty: {}", states_set.len());
    }

    let result = b.finish();
    assert!(result.is_deterministic());
    Ok(result)
}

/// Returns true if the given automaton accepts no strings.
pub fn is_empty(a: &Automaton) -> bool {
    if a.get_num_states() == 0 {
        // Common case: no states
        return true
    }

    if !a.is_accept(State(0)) && TransitionAccessor::get_num_transitions(a, State(0)) == 0 {
        // Common case: single initial state, no transitions
        return true
    }

    if a.is_accept(State(0)) {
        // Apparently common case: it accepts the empty string
        return false
    }

    let mut work_list = VecDeque::new();
    let mut seen = bitvec![0; a.get_num_states() as usize];
    work_list.push_back(State(0));
    seen.set(0, true);

    let mut t = Transition::default();
    while !work_list.is_empty() {
        let state = work_list.pop_front().unwrap();
        if a.is_accept(state) {
            return false;
        }

        let count = a.init_transition(state, &mut t);
        for i in 0..count {
            a.get_next_transition(&mut t);
            let dest = t.dest.unwrap();
            if !seen[dest.0 as usize] {
                work_list.push_back(dest);
                seen.set(dest.0 as usize, true);
            }
        }
    }

    true
}

/// Returns true if the given automaton accepts all strings. The automaton must be minimized.
pub fn is_total(a: &Automaton) -> bool {
    is_total_range(a, 0, char::MAX as u32)
}

/// Returns true if the given automaton accepts all strings for the specified min/max range of the
/// alphabet. The automaton must be minimized.
pub fn is_total_range(a: &Automaton, min_alphabet: u32, max_alphabet: u32) -> bool {
    if a.is_accept(State(0)) && TransitionAccessor::get_num_transitions(&a, State(0)) == 1 {
        let mut t = Transition::default();
        a.get_transition(State(0), 0, &mut t);
        t.dest == Some(State(0)) && t.min == min_alphabet && t.max == max_alphabet
    } else {
        false
    }
}

/// Returns true if the given string is accepted by the automaton. The input must be deterministic.
///
/// Complexity: linear in the length of the string.
/// 
/// # Notes
/// 
/// For full performance, use the [RunAutomaton] struct.
pub fn run(a: &Automaton, s: &str) -> bool {
    assert!(a.is_deterministic());
    let mut state = State(0);
    
    for c in s.chars() {
        let next_state = a.step(state, c as u32);

        state = match next_state {
            Some(s) => s,
            None => return false,
        };
    }

    a.is_accept(state)
}

/// Returns true if the given string (expressed as unicode codepoints) is accepted by the
/// automaton. The input must be deterministic.
///
/// Complexity: linear in the length of the string.
/// 
/// # Notes
/// 
/// For full performance, use the [RunAutomaton] struct.
pub fn run_codepoints(a: &Automaton, s: &[u32]) -> bool {
    assert!(a.is_deterministic());
    let mut state = State(0);
    
    for &c in s {
        let next_state = a.step(state, c);

        state = match next_state {
            Some(s) => s,
            None => return false,
        };
    }

    a.is_accept(state)
}

/// Returns the set of live states. A state is "live" if an accept state is reachable from it and
/// if it is reachable from the initial state.
fn get_live_states(a: &Automaton) -> BitVec {
    let mut live = get_live_states_from_initial(a);
    live &= get_live_states_to_accept(a);
    live
}

/// Returns [BitVec] marking states reachable from the initial state.
fn get_live_states_from_initial(a: &Automaton) -> BitVec {
    let num_states = a.get_num_states();
    let mut live = bitvec![0; num_states as usize];
    if num_states == 0 {
        return live;
    }

    let mut work_list = VecDeque::new();
    live.set(0, true);
    work_list.push_back(State(0));

    let mut t = Transition::default();
    while !work_list.is_empty() {
        let s = work_list.pop_front().unwrap();
        let count = a.init_transition(s, &mut t);
        for i in 0..count {
            a.get_next_transition(&mut t);
            let dest = t.dest.unwrap();
            if !live[dest.0 as usize] {
                live.set(dest.0 as usize, true);
                work_list.push_back(dest);
            }
        }
    }

    live
}


/// Returns [BitVec] marking states that can reach an accept state.
fn get_live_states_to_accept(a: &Automaton) -> BitVec {
    let mut builder = Builder::new();

    // NOTE: not quite the same thing as what SpecialOperations.reverse does:
    let mut t = Transition::default();
    let num_states = a.get_num_states();
    
    for s in 0..num_states {
        builder.create_state();
    }

    for s in 0..num_states {
        let count = a.init_transition(State(s), &mut t);
        for i in 0..count {
            a.get_next_transition(&mut t);
            builder.add_transition_range(t.dest.unwrap(), State(s), t.min, t.max);
        }
    }

    let a2 = builder.finish();
    let mut work_list =     VecDeque::new();
    let live = bitvec![0; num_states as usize];
    let accept_bits = a.get_accept_states();
    for s in accept_bits.iter_ones() {
        work_list.push_back(State(s as u32));
    }

    while ! work_list.is_empty() {
        let s = work_list.pop_front().unwrap();
        let count = a2.init_transition(s, &mut t);
        for i in 0..count {
            a2.get_next_transition(&mut t);
            let dest = t.dest.unwrap();
            if !live[dest.0 as usize] {
                live.set(dest.0 as usize, true);
                work_list.push_back(dest);
            }
        }
    }

    live
}

/// Removes transitions to dead states (a state is "dead" if it is not reachable from the initial
/// state or no accept state is reachable from it.)
pub fn remove_dead_states(a: &Automaton) -> Automaton {
    let num_states = a.get_num_states();
    let live_set = get_live_states(a);

    let map = vec![State(0); num_states as usize];
    let mut result = Automaton::default();
    
    for i in 0..num_states {
        if live_set[i as usize] {
            let state = result.create_state();
            map.as_mut_slice()[i as usize] = state;
            result.set_accept(state, a.is_accept(State(i)));
        
        }
    }

    let t = Transition::default();
    for i in 0..num_states {
        if live_set[i as usize] {
            let num_transitions = a.init_transition(State(i), &mut t);
        
            // filter out transitions to dead states:
            for j in 0..num_transitions {
                a.get_next_transition(&mut t);
                let dest = t.dest.unwrap();
                if live_set[dest.usize()] {
                    result.add_transition_range(map[i as usize], map[dest.usize()], t.min, t.max);
                }
            }
        }
    }

    result.finish_state();
    assert!(!has_dead_states(&result));
    result
}

/// Returns the longest string that is a prefix of all accepted strings and visits each state at
/// most once. The automaton must not have dead states. If this automaton has already been
/// converted to UTF-8 (e.g. using [UTF32ToUTF8]) then you should use [getCommonPrefixBytesRef] instead.
///
/// # Panics
/// Panics if the automaton has dead states reachable from the initial state.
/// 
/// # Returns
/// Returns the common prefix, which can be an empty (length 0) String
pub fn get_common_prefix(a: &Automaton) -> String {
    if has_dead_states_from_initial(a) {
        panic!("input automaton has dead states");
    }

    if is_empty(a) {
        return String::new();
    }

    let mut builder = String::new();
    let mut scratch = Transition::default();
    let visited = bitvec![0; a.get_num_states() as usize];
    let current = bitvec![0; a.get_num_states() as usize];
    let next = bitvec![0; a.get_num_states() as usize];

    current.set(0, true); // start with initial state.

    'algorithm: loop {
        let mut label = None;

        // do a pass, stepping all current paths forward once.
        for state in current.iter_ones() {
            visited.set(state, true);

            // if it is an accept state, we are done.
            if a.is_accept(State(state as u32)) {
                break 'algorithm;
            }

            for transition in 0..TransitionAccessor::get_num_transitions(a, State(state as u32)) {
                a.get_transition(State(state as u32), transition, &mut scratch);
                if label.is_none() {
                    label = Some(scratch.min);
                }

                // either a range of labels, or label that doesn't match all the other paths this round
                if scratch.min != scratch.max || Some(scratch.min) != label {
                    break 'algorithm;
                }

                // mark target state for next iteration
                next.set(scratch.dest.unwrap().0 as usize, true);
            }
        }

        assert!(label.is_some(), "we should not get here since we checked no dead-states up front?!");

        // add the label to the prefix.
        let c = char::from_u32(label.unwrap()).expect("invalid codepoint");
        builder.push(c);

        // swap current with next, clear next.
        swap(&mut current, &mut next);
        next.clear();
        next.resize(a.get_num_states() as usize, false);
    }

    builder
}

/// If this automaton accepts a single input, return it. Else, return null. The automaton must be
/// deterministic.
pub fn get_singleton(a: &Automaton) -> Option<Vec<u32>> {
    if !a.is_deterministic() {
        panic!("input automaton must be deterministic");
    }

    let mut builder = Vec::new();
    let mut visited = HashSet::new();
    let mut t = Transition::default();
    let mut s = State(0);

    loop {
        visited.insert(s);
        if !a.is_accept(s) {
            if TransitionAccessor::get_num_transitions(a, s) == 1 {
                a.get_transition(s, 0, &mut t);
                if t.min == t.max && !visited.contains(&t.dest.unwrap()) {
                    builder.push(t.min);
                    s = t.dest.unwrap();
                    continue;
                }
            }
        } else if TransitionAccessor::get_num_transitions(a, s) == 0 {
            return Some(builder);
        }

        // Automaton accepts more than one string:
        return None;
    }
}