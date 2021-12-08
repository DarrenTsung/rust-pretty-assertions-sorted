# Pretty Assertions (Sorted)

This crate wraps the [pretty_assertions](https://raw.githubusercontent.com/colin-kiegel/rust-pretty-assertions) crate, which highlights differences
in a test failure via a colorful diff.

However, the diff is based on the Debug output of the objects. For objects that
have non-deterministic output, eg. two HashMap with the same contents, the diff
will be polluted and obscured with with false-positive differences like here:

![standard assertion](https://raw.githubusercontent.com/DarrenTsung/rust-pretty-assertions-sorted/fe860f070bdfb29a399a32ff9d3b98ca8d958326/images/non_deterministic.png)

This is much easier to understand when the diff is sorted:

![sorted assertion](https://raw.githubusercontent.com/DarrenTsung/rust-pretty-assertions-sorted/fe860f070bdfb29a399a32ff9d3b98ca8d958326/images/sorted.png)

This is a pretty trivial example, you could solve this instead by converting the HashMap to
a BTreeMap in your tests. But it's not always feasible to replace the types with ordered
versions, especially for HashMaps that are deeply nested in types outside of your control.

To use the sorted version, import like this:

```rust
use pretty_assertions_sorted::{assert_eq, assert_eq_sorted};
```

`assert_eq` is provided as a re-export of `pretty_assertions::assert_eq` and should
be used if you don't want the Debug output to be sorted, or if the Debug output can't
be sorted (not supported types, eg. f64::NEG_INFINITY, or custom Debug output).

### Tip

Specify it as [`[dev-dependencies]`](http://doc.crates.io/specifying-dependencies.html#development-dependencies)
and it will only be used for compiling tests, examples, and benchmarks.
This way the compile time of `cargo build` won't be affected!

License: MIT/Apache-2.0
