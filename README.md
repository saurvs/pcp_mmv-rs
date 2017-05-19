# acme-rs

This is a Rust port of the Acme MMV sample application found in [pcp/src/pmdas/mmv/acme.c](https://github.com/performancecopilot/pcp/blob/master/src/pmdas/mmv/acme.c). It requires `libpcp` and `libpcp_mmv` to be installed.

Pre-generated bindings are provided and tracked in the repository. To generate them again, install `bindgen` through `cargo` and run `./generate-bindings.sh`. On my machine (Ubuntu 16.04.1), I had to install the `libclang-dev` *and* `clang` packages through `apt-get` for `bindgen` to run.

This port was writen during the community bonding period of my [GSoC project](https://medium.com/@saurvs/gsoc-2017-introduction-834825fb2aee) in order to decide between a Rust-C FFI approach vs. a pure Rust approach with regards to writing an MMV file.