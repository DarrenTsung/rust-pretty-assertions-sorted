//! # Pretty Assertions (Sorted)
//!
//! This crate wraps the [pretty_assertions](https://raw.githubusercontent.com/colin-kiegel/rust-pretty-assertions) crate, which highlights differences
//! in a test failure via a colorful diff.
//!
//! However, the diff is based on the Debug output of the objects. For objects that
//! have non-deterministic output, eg. two HashMap with the same contents, the diff
//! will be polluted and obscured with with false-positive differences like here:
//!
//! ![standard assertion](https://raw.githubusercontent.com/DarrenTsung/rust-pretty-assertions-sorted/fe860f070bdfb29a399a32ff9d3b98ca8d958326/images/non_deterministic.png)
//!
//! This is much easier to understand when the diff is sorted:
//!
//! ![sorted assertion](https://raw.githubusercontent.com/DarrenTsung/rust-pretty-assertions-sorted/fe860f070bdfb29a399a32ff9d3b98ca8d958326/images/sorted.png)
//!
//! This is a pretty trivial example, you could solve this instead by converting the HashMap to
//! a BTreeMap in your tests. But it's not always feasible to replace the types with ordered
//! versions, especially for HashMaps that are deeply nested in types outside of your control.
//!
//! To use the sorted version, import like this:
//!
//! ```rust
//! use pretty_assertions_sorted::{assert_eq, assert_eq_sorted};
//! ```
//!
//! `assert_eq` is provided as a re-export of `pretty_assertions::assert_eq` and should
//! be used if you don't want the Debug output to be sorted, or if the Debug output can't
//! be sorted (not supported types, eg. f64::NEG_INFINITY, or custom Debug output).
//!
//! ## Tip
//!
//! Specify it as [`[dev-dependencies]`](http://doc.crates.io/specifying-dependencies.html#development-dependencies)
//! and it will only be used for compiling tests, examples, and benchmarks.
//! This way the compile time of `cargo build` won't be affected!
use std::fmt;

use darrentsung_debug_parser::{parse, Value};
pub use pretty_assertions::{assert_eq, assert_ne, Comparison};

/// This is a wrapper with similar functionality to [`assert_eq`], however, the
/// [`Debug`] representation is sorted to provide deterministic output.
///
/// Not all [`Debug`] representations are sortable yet and this doesn't work with
/// custom [`Debug`] implementations that don't conform to the format that #[derive(Debug)]
/// uses, eg. `fmt.debug_struct()`, `fmt.debug_map()`, etc.
///
/// Don't use this if you want to test the ordering of the types that are sorted, since
/// sorting will clobber any previous ordering.
///
/// Potential use-cases that aren't implemented yet:
/// * Blocklist for field names that shouldn't be sorted
/// * Sorting more than just maps (struct fields, lists, etc.)
#[macro_export]
macro_rules! assert_eq_sorted {
    ($left:expr, $right:expr$(,)?) => ({
        $crate::assert_eq_sorted!(@ $left, $right, "", "");
    });
    ($left:expr, $right:expr, $($arg:tt)*) => ({
        $crate::assert_eq_sorted!(@ $left, $right, ": ", $($arg)+);
    });
    (@ $left:expr, $right:expr, $maybe_semicolon:expr, $($arg:tt)*) => ({
        match (&($left), &($right)) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    ::core::panic!("assertion failed: `(left == right)`{}{}\
                       \n\
                       \n{}\
                       \n",
                       $maybe_semicolon,
                       format_args!($($arg)*),
                       $crate::Comparison::new(&SortedDebug(left_val), &SortedDebug(right_val))
                    )
                }
            }
        }
    });
}

/// New-type wrapper around an object that sorts the fmt::Debug output
/// when displayed for deterministic output.
///
/// This works through parsing the output and sorting the `debug_map()`
/// type.
///
/// Potential use-cases that aren't implemented yet:
/// * Blocklist for field names that shouldn't be sorted
/// * Sorting more than just maps (struct fields, lists, etc.)
pub struct SortedDebug<T>(T);

impl<T: fmt::Debug> fmt::Debug for SortedDebug<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut value = match parse(&format!("{:?}", self.0)) {
            Ok(value) => value,
            Err(err) => {
                ::core::panic!("Failed to parse Debug output, err: {}", err)
            }
        };

        sort_maps(&mut value);

        fmt::Debug::fmt(&value, f)
    }
}

fn sort_maps(v: &mut Value) {
    match v {
        Value::Struct(s) => {
            for ident_value in &mut s.values {
                sort_maps(&mut ident_value.value);
            }
        }
        Value::Set(s) => {
            for child_v in &mut s.values {
                sort_maps(child_v);
            }
        }
        Value::Map(map) => {
            map.values.sort_by(|a, b| a.key.cmp(&b.key));

            for key_value in &mut map.values {
                sort_maps(&mut key_value.key);
                sort_maps(&mut key_value.value);
            }
        }
        Value::List(l) => {
            for child_v in &mut l.values {
                sort_maps(child_v);
            }
        }
        Value::Tuple(t) => {
            for child_v in &mut t.values {
                sort_maps(child_v);
            }
        }
        // No need to recurse for Term variant.
        Value::Term(_) => (),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use std::assert_eq;
    use std::collections::HashMap;

    const TEST_RERUNS_FOR_DETERMINISM: u32 = 100;

    fn sorted_debug<T: fmt::Debug>(v: T) -> String {
        format!("{:#?}", SortedDebug(v))
    }

    #[test]
    fn noop_sorts() {
        for _ in 0..TEST_RERUNS_FOR_DETERMINISM {
            assert_eq!(sorted_debug(2), "2");
        }
    }

    #[test]
    fn sorts_hashmap() {
        for _ in 0..TEST_RERUNS_FOR_DETERMINISM {
            // Note that we have to create the HashMaps each try
            // in order to induce non-determinism in the debug output.
            let item = {
                let mut map = HashMap::new();
                map.insert(1, true);
                map.insert(2, true);
                map.insert(20, true);
                map
            };

            let expected = indoc!(
                "{
                    1: true,
                    2: true,
                    20: true,
                }"
            );
            assert_eq!(sorted_debug(item), expected);
        }
    }

    #[test]
    fn sorts_object_with_hashmap() {
        #[derive(Debug)]
        struct Foo {
            bar: Bar,
        }

        #[derive(Debug)]
        struct Bar {
            count: HashMap<&'static str, Zed>,
            value: usize,
        }

        #[derive(Debug)]
        struct Zed;

        for _ in 0..TEST_RERUNS_FOR_DETERMINISM {
            let item = Foo {
                bar: Bar {
                    count: {
                        let mut map = HashMap::new();
                        map.insert("hello world", Zed);
                        map.insert("lorem ipsum", Zed);
                        map
                    },
                    value: 200,
                },
            };

            let expected = indoc!(
                "Foo {
                    bar: Bar {
                        count: {
                            \"hello world\": Zed,
                            \"lorem ipsum\": Zed,
                        },
                        value: 200,
                    },
                }"
            );
            assert_eq!(sorted_debug(item), expected);
        }
    }

    #[test]
    fn hashmap_with_object_values() {
        #[derive(Debug)]
        struct Foo {
            value: f32,
            bar: Vec<Bar>,
        }

        #[derive(Debug)]
        struct Bar {
            elo: i32,
        }

        for _ in 0..TEST_RERUNS_FOR_DETERMINISM {
            let item = {
                let mut map = HashMap::new();
                map.insert(
                    "foo",
                    Foo {
                        value: 12.2,
                        bar: vec![Bar { elo: 200 }, Bar { elo: -12 }],
                    },
                );
                map.insert(
                    "foo2",
                    Foo {
                        value: -0.2,
                        bar: vec![],
                    },
                );
                map
            };

            let expected = indoc!(
                "{
                    \"foo\": Foo {
                        value: 12.2,
                        bar: [
                            Bar {
                                elo: 200,
                            },
                            Bar {
                                elo: -12,
                            },
                        ],
                    },
                    \"foo2\": Foo {
                        value: -0.2,
                        bar: [],
                    },
                }"
            );
            assert_eq!(sorted_debug(item), expected);
        }
    }

    #[test]
    fn hashmap_with_object_keys() {
        #[derive(Debug, PartialEq, Eq, Hash)]
        struct Foo {
            value: i32,
            bar: Vec<Bar>,
        }

        #[derive(Debug, PartialEq, Eq, Hash)]
        struct Bar {
            elo: i32,
        }

        for _ in 0..TEST_RERUNS_FOR_DETERMINISM {
            let item = {
                let mut map = HashMap::new();
                map.insert(
                    Foo {
                        value: 12,
                        bar: vec![Bar { elo: 200 }, Bar { elo: -12 }],
                    },
                    "foo",
                );
                map.insert(
                    Foo {
                        value: -2,
                        bar: vec![],
                    },
                    "foo2",
                );
                map
            };

            let expected = indoc!(
                "{
                    Foo {
                        value: -2,
                        bar: [],
                    }: \"foo2\",
                    Foo {
                        value: 12,
                        bar: [
                            Bar {
                                elo: 200,
                            },
                            Bar {
                                elo: -12,
                            },
                        ],
                    }: \"foo\",
                }"
            );
            assert_eq!(sorted_debug(item), expected);
        }
    }
}
