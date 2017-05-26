#!/bin/bash
bindgen wrapper.h --constified-enum mmv_stats_flags --no-unstable-rust --blacklist-type max_align_t -o src/sys.rs
