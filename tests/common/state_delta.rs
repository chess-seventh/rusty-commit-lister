/// State-delta assertion port for rusty-commit-lister (Rust).
///
/// Bootstrapped by DISTILL wave 2026-05-18 (first DISTILL in this project).
/// Language: Rust — polyglot adapter matrix entry.
///
/// Universe assertion contract: every state-mutating test at layers 1-3 calls
/// `assert_state_delta(before, after, universe, expected)`. Universe entries are
/// port-exposed names only (never internal struct fields). Anything in universe
/// not in expected MUST remain unchanged — fail-closed.
///
/// Predicate library: `set_to`, `unchanged`, `appended_with`, `containing`.
/// Additional predicates added lazily as needed.
use std::collections::HashMap;

/// A snapshot of observable state keyed by port-exposed name.
pub type StateSnapshot = HashMap<String, StateValue>;

/// An observable value at a port boundary.
#[derive(Debug, Clone, PartialEq)]
pub enum StateValue {
    Int(i64),
    Usize(usize),
    Bool(bool),
    Str(String),
    Strings(Vec<String>),
    None,
}

/// A predicate over a state value transition (before → after).
pub enum Predicate {
    SetTo(StateValue),
    Unchanged,
    AppendedWith(String),
    Containing(String),
}

impl Predicate {
    pub fn check(&self, before: Option<&StateValue>, after: Option<&StateValue>, key: &str) {
        match self {
            Predicate::SetTo(expected) => {
                let actual = after.unwrap_or(&StateValue::None);
                assert_eq!(
                    actual, expected,
                    "state_delta: key '{}' expected SetTo({:?}), got {:?}",
                    key, expected, actual
                );
            }
            Predicate::Unchanged => {
                assert_eq!(
                    before, after,
                    "state_delta: key '{}' expected Unchanged, before={:?} after={:?}",
                    key, before, after
                );
            }
            Predicate::AppendedWith(item) => {
                if let Some(StateValue::Strings(after_vec)) = after {
                    assert!(
                        after_vec.contains(item),
                        "state_delta: key '{}' expected AppendedWith({:?}), after={:?}",
                        key,
                        item,
                        after_vec
                    );
                } else {
                    panic!(
                        "state_delta: key '{}' expected AppendedWith but after={:?}",
                        key, after
                    );
                }
            }
            Predicate::Containing(substr) => {
                if let Some(StateValue::Str(s)) = after {
                    assert!(
                        s.contains(substr.as_str()),
                        "state_delta: key '{}' expected Containing({:?}), got {:?}",
                        key,
                        substr,
                        s
                    );
                } else {
                    panic!(
                        "state_delta: key '{}' expected Containing but after={:?}",
                        key, after
                    );
                }
            }
        }
    }
}

/// Predicate constructors.
pub fn set_to(v: StateValue) -> Predicate {
    Predicate::SetTo(v)
}
pub fn unchanged() -> Predicate {
    Predicate::Unchanged
}
pub fn appended_with(s: impl Into<String>) -> Predicate {
    Predicate::AppendedWith(s.into())
}
pub fn containing(s: impl Into<String>) -> Predicate {
    Predicate::Containing(s.into())
}

/// Assert observable state delta.
///
/// - `universe`: the set of port-exposed names this test promises to track.
/// - `expected`: which universe entries change and how.
/// - Any entry in `universe` that is NOT in `expected` must be unchanged (fail-closed).
///
/// # Panics
///
/// Panics with a descriptive message if any predicate fails or if an unexpected
/// mutation is detected on a universe entry not in `expected`.
pub fn assert_state_delta(
    before: &StateSnapshot,
    after: &StateSnapshot,
    universe: &[&str],
    expected: HashMap<&str, Predicate>,
) {
    for key in universe {
        let before_val = before.get(*key);
        let after_val = after.get(*key);

        if let Some(pred) = expected.get(key) {
            pred.check(before_val, after_val, key);
        } else {
            // Not in expected → must be unchanged (fail-closed)
            assert_eq!(
                before_val, after_val,
                "state_delta: unexpected mutation on universe key '{}' \
                 (not in expected map). before={:?}, after={:?}. \
                 Add an explicit predicate for this key.",
                key, before_val, after_val
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_to_passes_when_value_matches() {
        let before: StateSnapshot = HashMap::new();
        let mut after: StateSnapshot = HashMap::new();
        after.insert("exit_code".to_string(), StateValue::Int(0));

        assert_state_delta(
            &before,
            &after,
            &["exit_code"],
            [("exit_code", set_to(StateValue::Int(0)))].into(),
        );
    }

    #[test]
    fn unchanged_passes_when_value_is_same() {
        let mut before: StateSnapshot = HashMap::new();
        before.insert("mode".to_string(), StateValue::Str("Browse".to_string()));
        let after = before.clone();

        assert_state_delta(&before, &after, &["mode"], [("mode", unchanged())].into());
    }

    #[test]
    #[should_panic(expected = "unexpected mutation")]
    fn fail_closed_detects_unexpected_mutation() {
        let mut before: StateSnapshot = HashMap::new();
        before.insert("cursor".to_string(), StateValue::Usize(0));
        let mut after: StateSnapshot = HashMap::new();
        after.insert("cursor".to_string(), StateValue::Usize(1));

        // "cursor" is in universe but NOT in expected → must be unchanged → panics
        assert_state_delta(
            &before,
            &after,
            &["cursor"],
            HashMap::new(), // empty expected — fail-closed
        );
    }
}
