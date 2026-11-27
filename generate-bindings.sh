#!/usr/bin/env bash
#
# Regenerates src/sys.rs from the PCP headers installed on this machine.
#
# Requirements:
#   - PCP development headers (Debian/Ubuntu: libpcp3-dev libpcp-mmv1-dev)
#   - libclang (Debian/Ubuntu: libclang-dev)
#   - bindgen-cli: cargo install bindgen-cli
#
# The committed bindings were generated on 64-bit little-endian Linux.
# CI regenerates them and fails if they differ from the committed file.
set -euo pipefail
cd "$(dirname "$0")"

bindgen wrapper.h \
    --output src/sys.rs \
    --rust-target 1.85 \
    --rust-edition 2024 \
    --use-core \
    --merge-extern-blocks \
    --with-derive-default \
    --no-prepend-enum-name \
    --default-macro-constant-type signed \
    --allowlist-function 'mmv_.*' \
    --allowlist-type 'mmv_.*' \
    --allowlist-type 'pmUnits' \
    --allowlist-type 'pmAtomValue' \
    --allowlist-var 'MMV_.*' \
    --allowlist-var 'PM_(SPACE|TIME|COUNT)_.*'
