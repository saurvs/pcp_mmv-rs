#!/bin/bash
bindgen wrapper.h --no-unstable-rust --blacklist-type max_align_t -o src/pcp_mmv.rs
