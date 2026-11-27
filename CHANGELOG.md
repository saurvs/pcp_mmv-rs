# Changelog

## 0.2.0 - 2026-11-27

Refresh of the 2017 proof of concept so that it builds and runs with current
Rust and PCP. For production use, see
[hornet](https://github.com/performancecopilot/hornet).

### Changed (breaking)

- Rust 2024 edition; the minimum supported Rust version is 1.85.
- The bindings in `src/sys.rs` were regenerated with bindgen 0.73 and limited
  to the MMV API (`mmv_*`, `pmUnits`, `pmAtomValue` and the `PM_SPACE_*`,
  `PM_TIME_*` and `PM_COUNT_*` scales). They are now in the public `sys`
  module instead of being re-exported from the crate root.
- The safe wrapper is now sound:
  - `Metric`, `Indom` and `Instance` own their data, instead of being the C
    structs with raw pointers to leaked strings and caller-owned arrays.
  - `Mmv::start` returns `io::Result<Mmv>` instead of storing a null handle
    on failure, and dropping an `Mmv` calls `mmv_stats_stop`.
  - `Mmv::lookup_value` takes `Option<&str>` for the instance and returns
    `Option<AtomValue>`, which borrows the `Mmv`, instead of a value that
    could be null or outlive the mapping.
  - `Mmv` is `Send` but no longer `Sync`, because `libpcp_mmv` does not
    update values atomically.
  - Names that do not fit in 63 bytes now panic instead of being written
    without a terminating NUL byte.
  - Names now use `c_char`, so the crate builds on aarch64 Linux.
- `MMV` is renamed to `Mmv`, and `MMV::inc_value(&mut value, x)` is now
  `AtomValue::inc(x)`. `AtomValue` also gains `set`, `set_string` and
  `ull`/`set_ull`/`inc_ull` take `&self`.
- `MetricType` and `MetricSem` are Rust enums (`MetricType::U64`,
  `MetricSem::Counter`, ...), `PMUnits` is now `Units` with a `Units::new`
  constructor like PCP's `MMV_UNITS`, and flags are a `Flags` bitflags type.
- `inotify`, `nickel`, `rand` and `sha2` are no longer dependencies of the
  library; the examples use `inotify` 0.11, `rand` 0.10 and `std::net`.

### Fixed

- The Acme example now matches PCP's `acme.c`: it records machine and queue
  time in microseconds instead of adding 1 per product.
- The `file` example used item 7 for both of its metrics.

### Added

- Integration tests, GitHub Actions CI on x86_64 and aarch64 Linux (stable
  and 1.85) that also runs the examples and checks their output with
  `mmvdump`, and a check that the checked-in bindings are up to date.
- `PCP_LIB_DIR` to link against a `libpcp_mmv` outside the default search
  path.
- License: MIT OR Apache-2.0.

## 0.1.0 - 2017-05-26

- Initial proof of concept: bindings, safe wrappers, and the `acme`, `http`
  and `file` examples.
