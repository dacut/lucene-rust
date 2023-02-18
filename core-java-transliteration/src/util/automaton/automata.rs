//! Construction of basic automata.
use {
    crate::util::automaton::{
        automaton::{Automaton, Builder},
        daciuk_mihov_automaton_builder,
        operations,
        state::State,
    },
    std::{cmp::Ordering, io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult}},
};

/// Returns a new (deterministic) automaton with the empty language.
pub fn make_empty() -> Automaton {
    let mut a = Automaton::default();
    a.finish_state();
    a
}

/// Returns a new (deterministic) automaton that accepts only the empty string.
pub fn make_empty_string() -> Automaton {
    let mut a = Automaton::default();
    a.create_state();
    a.set_accept(State(0), true);
    a
}

/// Returns a new (deterministic) automaton that accepts all strings.
pub fn make_any_string() -> Automaton {
    let mut a = Automaton::default();
    let s = a.create_state();
    a.set_accept(s, true);
    a.add_transition_range(s, s, 0, char::MAX as u32);
    a.finish_state();
    a
}

/// Returns a new (deterministic) automaton that accepts all binary terms.
pub fn make_any_binary() -> Automaton {
    let mut a = Automaton::default();
    let s = a.create_state();
    a.set_accept(s, true);
    a.add_transition_range(s, s, 0, 255);
    a.finish_state();
    a
}

/// Returns a new (deterministic) automaton that accepts all binary terms except the empty string.
pub fn make_non_empty_binary() -> Automaton {
    let mut a = Automaton::default();
    let s1 = a.create_state();
    let s2 = a.create_state();
    a.set_accept(s2, true);
    a.add_transition_range(s1, s2, 0, 255);
    a.add_transition_range(s2, s2, 0, 255);
    a.finish_state();
    a
}

/// Returns a new (deterministic) automaton that accepts any single codepoint.
pub fn make_any_char() -> Automaton {
    let mut a = Automaton::default();
    let s = a.create_state();
    a.set_accept(s, true);
    a.add_transition_range(s, s, 0, char::MAX as u32);
    a.finish_state();
    a
}

/// Accept any single character starting from the specified state, returning the new state
pub fn append_any_char(a: &mut Automaton, state: State) -> State {
    let new_state = a.create_state();
    a.add_transition_range(state, new_state, 0, char::MAX as u32);
    new_state
}

/// Returns a new (deterministic) automaton that accepts a single codepoint of the given value.
pub fn make_char(c: char) -> Automaton {
    make_char_range(c, c)
}

/// Appends the specified character to the specified state, returning a new state.
pub fn append_char(a: &mut Automaton, state: State, c: char) -> State {
    let new_state = a.create_state();
    a.add_transition_range(state, new_state, c as u32, c as u32);
    new_state
}

/// Returns a new (deterministic) automaton that accepts a single codepoint whose value is in the
/// given interval (including both end points).
pub fn make_char_range(min: char, max: char) -> Automaton {
    if min > max {
        return make_empty();
    }

    let mut a = Automaton::default();
    let s1 = a.create_state();
    let s2 = a.create_state();
    a.set_accept(s2, true);
    a.add_transition_range(s1, s2, min as u32, max as u32);
    a.finish_state();
    a
}

/// Constructs sub-automaton corresponding to decimal numbers of length `x[n..].len()`.
fn any_of_right_length(builder: &mut Builder, x: &str, n: usize) -> State {
    let s = builder.create_state();
    if x.len() == n {
        builder.set_accept(s, true);
    } else {
        builder.add_transition_range(s, any_of_right_length(builder, x, n + 1), '0' as u32, '9' as u32);
    }

    s
}

/// Constructs sub-automaton corresponding to decimal numbers of value at least `x[n..]` and
/// length `x[n..].len()`.
///
/// # Panics
///
/// Panics if `n` is not a valid character index into `x`.
fn at_least(builder: &mut Builder, x: &str, n: usize, initials: &mut Vec<State>, zeros: bool) -> State {
    let s = builder.create_state();
    if x.len() == n {
        builder.set_accept(s, true);
    } else {
        if zeros {
            initials.push(s);
        }

        let c = x[n..].chars().next().unwrap();
        builder.add_transition(s, at_least(builder, x, n + 1, initials, zeros && c == '0'), c as u32);

        if c < '9' {
            builder.add_transition_range(s, any_of_right_length(builder, x, n + 1), c as u32 + 1, '9' as u32);
        }
    }

    s
}

/// Constructs sub-automaton corresponding to decimal numbers of value at most `x[n..] and
/// length `x[n..].len()`.
///
/// # Panics
///
/// Panics if `n` is not a valid character index into `x`.
fn at_most(builder: &mut Builder, x: &str, n: usize) -> State {
    let s = builder.create_state();
    if x.len() == n {
        builder.set_accept(s, true);
    } else {
        let c = x[n..].chars().next().unwrap();
        builder.add_transition(s, at_most(builder, x, n + 1), c as u32);
        if c > '0' {
            builder.add_transition_range(s, any_of_right_length(builder, x, n + 1), '0' as u32, c as u32 - 1);
        }
    }

    s
}

/// Constructs sub-automaton corresponding to decimal numbers of value between `x[n..]` and
/// `y[n..]` and of length `x[n..].len()` (which must be equal to
/// `y[n..].len()`).
///
/// # Panics
///
/// Panics if `n` is not a valid character index into both `x` and `y`.
fn between(builder: &mut Builder, x: &str, y: &str, n: usize, initials: &mut Vec<State>, zeros: bool) -> State {
    let s = builder.create_state();
    if x.len() == n {
        builder.set_accept(s, true);
    } else {
        if zeros {
            initials.push(s);
        }

        let cx = x[n..].chars().next().unwrap();
        let cy = y[n..].chars().next().unwrap();

        if cx == cy {
            builder.add_transition(s, between(builder, x, y, n + 1, initials, zeros && cx == '0'), cx as u32);
        } else {
            // cx < cy
            builder.add_transition(s, at_least(builder, x, n + 1, initials, zeros && cx == '0'), cx as u32);
            builder.add_transition(s, at_most(builder, y, n + 1), cy as u32);

            if cx as u32 + 1 < cy as u32 {
                builder.add_transition_range(s, any_of_right_length(builder, x, n + 1), cx as u32 + 1, cy as u32 - 1);
            }
        }
    }

    s
}

fn suffix_is_zeros(br: &[u8], pos: usize) -> bool {
    for i in pos..br.len() {
        if br[i] != 0 {
            return false;
        }
    }

    true
}

/// Creates a new deterministic, minimal automaton accepting all binary terms in the specified
/// interval. Note that unlike [make_decimal_interval], the returned automaton is infinite,
/// because terms behave like floating point numbers leading with a decimal point. However, in the
/// special case where `min == max`, and both are inclusive, the automata will be finite and accept
/// exactly one term.
pub fn make_binary_interval(
    min: Option<&[u8]>,
    min_inclusive: bool,
    max: Option<&[u8]>,
    max_inclusive: bool,
) -> IoResult<Automaton> {
    if min.is_none() && !min_inclusive {
        return Err(IoError::new(IoErrorKind::InvalidInput, "min_inclusive must be true if min is None (open ended)"));
    }

    if max.is_none() && !max_inclusive {
        return Err(IoError::new(IoErrorKind::InvalidInput, "max_inclusive must be true if max is None (open ended)"));
    }

    let min = match min {
        Some(min) => min,
        None => &[],
    };

    let cmp = if let Some(max) = max {
        min.cmp(max)
    } else {
        if min.len() == 0 {
            if min_inclusive {
                return Ok(make_any_binary());
            } else {
                return Ok(make_non_empty_binary());
            }
        }

        Ordering::Less
    };

    match cmp {
        Ordering::Equal => {
            if !min_inclusive || !max_inclusive {
                return Ok(make_empty());
            } else {
                return Ok(make_binary(min));
            }
        }
        Ordering::Greater => {
            // max < min
            return Ok(make_empty())
        }
        Ordering::Less => (),
    }

    if let Some(max) = max {
        if max.starts_with(min) && suffix_is_zeros(max, min.len()) {
            // Finite case: no sink state!
            let mut max_length = max.len();

            // the == case was handled above.
            assert!(max_length > min.len());

            // bar -> bar\0+
            if !max_inclusive {
                max_length -= 1;
            }

            if max_length == min.len() {
                if !min_inclusive {
                    return Ok(make_empty());
                } else {
                    return Ok(make_binary(min));
                }
            }

            let mut a = Automaton::default();
            let mut last_state = a.create_state();
            for i in 0..min.len() {
                let state = a.create_state();
                let label = min[i];
                a.add_transition(last_state, state, label as u32);
                last_state = state;
            }

            if min_inclusive {
                a.set_accept(last_state, true);
            }

            for i in min.len()..max_length {
                let state = a.create_state();
                a.add_transition(last_state, state, 0);
                a.set_accept(state, true);
                last_state = state;
            }

            a.finish_state();
            return Ok(a);
        }
    }

    let mut a = Automaton::default();
    let start_state = a.create_state();
    let sink_state = a.create_state();
    a.set_accept(sink_state, true);

    // This state accepts all suffixes:
    a.add_transition_range(sink_state, sink_state, 0, 255);

    let mut equal_prefix = true;
    let mut last_state = start_state;
    let mut first_max_state = None;
    let mut shared_prefix_length = 0;
    for i in 0..min.len() {
        let min_label = min[i];
        let max_label = None;
        if let Some(max) = max {
            if equal_prefix && i < max.len() {
                max_label = Some(max[i])
            }
        }

        let next_state = if min_inclusive && i == min.len() - 1 && (!equal_prefix || Some(min_label) != max_label) {
            sink_state
        } else {
            a.create_state()
        };
        
        if equal_prefix {
            if Some(min_label) == max_label {
                // Still in shared prefix
                a.add_transition(last_state, next_state, min_label as u32);
            } else if max.is_none() {
                equal_prefix = false;
                shared_prefix_length = 0;
                a.add_transition_range(last_state, sink_state, min_label as u32 + 1, 0xff);
                a.add_transition(last_state, next_state, min_label as u32);
            } else {
                // This is the first point where min & max diverge:
                let Some(max_label) = max_label else {
                    panic!("max_label should be Some");
                };
                assert!(max_label > min_label);
                a.add_transition(last_state, next_state, min_label as u32);
                
                if max_label as u32 > min_label as u32 + 1 {
                    a.add_transition_range(last_state, sink_state, min_label as u32 + 1, max_label as u32 - 1);
                }

                // Now fork off path for max: (max will be Some if max_inclusive is false)
                if max_inclusive || i < max.unwrap().len() - 1 {
                    let new_state = a.create_state();
                    first_max_state = Some(new_state);

                    // FIXME: This Option check doesn't exist in Java, which derefs max blindly (possibly unsafe?).
                    if let Some(max) = max {
                        if i < max.len() - 1 {
                            a.set_accept(new_state, true);
                        }
                    }

                    a.add_transition(last_state, new_state, max_label as u32);
                }

                equal_prefix = false;
                shared_prefix_length = i;
            }
        } else {
            // Ok, already diverged:
            a.add_transition(last_state, next_state, min_label as u32);
            if min_label < 0xff {
                a.add_transition_range(last_state, sink_state, min_label as u32 + 1, 255);
            }
        }

        last_state = next_state;
    }

    // Accept any suffix appended to the min term:
    if !equal_prefix && last_state != sink_state && last_state != start_state {
        a.add_transition_range(last_state, sink_state, 0, 255);
    }

    if min_inclusive {
        // Accept exactly the min term:
        a.set_accept(last_state, true);
    }

    if let Some(max) = max {
        // Now do max:
        match first_max_state {
            None => {
                // Min was a full prefix of max
                shared_prefix_length = min.len();
            }
            Some(first_max_state) => {
                last_state = first_max_state;
                shared_prefix_length += 1;
            }
        }

        for i in shared_prefix_length..max.len() {
            let max_label = max[i];
            if max_label > 0 {
                a.add_transition_range(last_state, sink_state, 0, max_label as u32 - 1);
            }

            if max_inclusive || i < max.len() - 1 {
                let next_state = a.create_state();
                if i < max.len() - 1 {
                    a.set_accept(next_state, true);
                }

                a.add_transition(last_state, next_state, max_label as u32);
                last_state = next_state;
            }
        }

        if max_inclusive {
            a.set_accept(last_state, true);
        }
    }

    a.finish_state();
    Ok(a)
}

/// Returns a new automaton that accepts strings representing decimal (base 10) non-negative
/// integers in the given interval.
///
/// # Parameters
/// * `min`: minimal value of interval
/// * `max`: maximal value of interval (both end points are included in the interval)
/// * `digits`: if > 0, use fixed number of digits (strings must be prefixed by 0's to obtain
///   the right length) - otherwise, the number of digits is not fixed (any number of leading 0s
///   is accepted)
/// 
/// # Errors
/// Returns [std::io::Error] with [std::io::ErrorKind::InvalidInput] if min < max or if numbers in the interval cannot be
/// expressed with the given fixed number of digits
pub fn make_decimal_interval(min: u8, max: u8, digits: usize) -> IoResult<Automaton> {
    let x = format!("{min}");
    let y = format!("{max}");

    if min > max {
        return Err(IoError::new(IoErrorKind::InvalidInput, "min must be less than or equal to max"));
    }

    if digits > 0 && y.len() > digits {
        return Err(IoError::new(IoErrorKind::InvalidInput, "max cannot be expressed with the given number of digits"));
    }

    let mut d = if digits > 0 {
        digits
    } else {
        y.len()
    };

    let mut bx = String::new();
    for i in x.len()..d {
        bx.push('0');
    }

    bx.push_str(&x);
    let x = bx;

    let mut by = String::new();
    for i in y.len()..d {
        by.push('0');
    }

    by.push_str(&y);
    let y = by;

    let mut builder = Builder::new();
    if digits == 0 {
        // Reserve the "real" initial state:
        builder.create_state();
    }

    let mut initials = Vec::new();
    between(&mut builder, &x, &y, 0, &mut initials, digits == 0);
    let a1 = builder.finish();

    if digits == 0 {
        a1.add_transition(State(0), State(0), 0);
        for p in initials {
            a1.add_epsilon(State(0), p);
        }
        a1.finish_state();
    }

    Ok(a1)

}

/// Returns a new (deterministic) automaton that accepts the single given string.
pub fn make_string(s: &str) -> Automaton {
    let mut a = Automaton::default();
    let mut last_state = a.create_state();
    for c in s.chars() {
        let state = a.create_state();
        a.add_transition(last_state, state, c as u32);
        last_state = state;
    }

    a.set_accept(last_state, true);
    a.finish_state();

    assert!(a.is_deterministic());
    assert!(!operations::has_dead_states(&a));

    a
}

/// Returns a new (deterministic) automaton that accepts the single given binary term.
pub fn make_binary(term: &[u8]) -> Automaton {
    let mut a = Automaton::default();
    let mut last_state = a.create_state();
    
    for b in term {
        let state = a.create_state();
        a.add_transition(last_state, state, *b as u32);
        last_state = state;
    }

    a.set_accept(last_state, true);
    a.finish_state();

    assert!(a.is_deterministic());
    assert!(!operations::has_dead_states(&a));

    a
}

/// Returns a new (deterministic) automaton that accepts the single given string from the specified
/// unicode code points.
pub fn make_string_from_chars(word: &[char]) -> Automaton {
    let mut a = Automaton::default();
    let mut last_state = a.create_state();
    for c in word {
        let state = a.create_state();
        a.add_transition(last_state, state, *c as u32);
        last_state = state;
    }

    a.set_accept(last_state, true);
    a.finish_state();

    assert!(a.is_deterministic());
    assert!(!operations::has_dead_states(&a));

    a
}

/// Returns a new (deterministic and minimal) automaton that accepts the union of the given
/// collection of [str]s.
///
/// # Parameters
/// * `strs`: The input strings, UTF-8 encoded. The collection must be in sorted order.
/// 
/// # Returns
/// An [Automaton] accepting all input strings. The resulting automaton is codepoint
/// based (full unicode codepoints on transitions).
pub fn make_string_union(strs: &[&str]) -> Automaton {
    if strs.is_empty() {
        make_empty()
    } else {
        daciuk_mihov_automaton_builder::build(strs)
    }
}
