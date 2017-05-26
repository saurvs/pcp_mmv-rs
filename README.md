# pcp_mmv

This is Rust binding to `libpcp_mmv`, with safe wrappers.

It also contains the following examples:
- a port of the Acme example found in [pcp/src/pmdas/mmv/acme.c](https://github.com/performancecopilot/pcp/blob/master/src/pmdas/mmv/acme.c)
- a program that starts an HTTP server on `localhost:6767`, and instruments the number of HTTP GET requests it recieves
- a (Linux-only) program that instruments the number of `CLOSE_WRITE` notifications it recieves from `inotify` w.r.t an input file path, and also writes the hash of file's contents to the MMV file. `nano` works best to test this example.

Pre-generated bindings are provided and tracked in the repository. To generate them again, install `bindgen` through `cargo` and run `./generate-bindings.sh`. On my machine (Ubuntu 16.04.1), I had to install the `libclang-dev` *and* `clang` packages through `apt-get` for `bindgen` to run.

This crate was writen during the community bonding period of my [GSoC project](https://medium.com/@saurvs/gsoc-2017-introduction-834825fb2aee) in order to decide between a Rust-C FFI approach vs. a pure Rust approach with regards to writing an MMV file.