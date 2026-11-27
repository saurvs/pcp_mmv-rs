#!/usr/bin/env bash
#
# Runs each example briefly, then checks the MMV file it wrote with PCP's
# mmvdump, which exits non-zero if the file is malformed.
set -euo pipefail

export PCP_TMP_DIR="${RUNNER_TEMP:-/tmp}/pcp_mmv-examples"
rm -rf "$PCP_TMP_DIR"
mkdir -p "$PCP_TMP_DIR/mmv"
trap 'jobs -p | xargs -r kill 2>/dev/null || true' EXIT

# find exits non-zero for unreadable directories, hence the "|| true".
mmvdump=$(command -v mmvdump ||
    find /usr/lib /usr/libexec /var/lib/pcp -name mmvdump -type f 2>/dev/null | head -n1 ||
    true)
if [ ! -x "$mmvdump" ]; then
    echo "mmvdump not found" >&2
    exit 1
fi
echo "Using $mmvdump"

cargo build --examples
examples=target/debug/examples

dump() {
    echo "--- mmvdump $1"
    "$mmvdump" "$PCP_TMP_DIR/mmv/$1" | tee "$PCP_TMP_DIR/$1.txt"
}

expect() {
    if ! grep -qF -- "$2" "$PCP_TMP_DIR/$1.txt"; then
        echo "expected '$2' in the mmvdump output for $1" >&2
        exit 1
    fi
}

wait_for_line() { # file, text
    for _ in $(seq 100); do
        grep -qF -- "$2" "$1" 2>/dev/null && return 0
        sleep 0.1
    done
    echo "timed out waiting for '$2' in $1" >&2
    exit 1
}

# acme: build 20 products, then check the per-product counts add up.
timeout 60 "$examples/acme" 20
dump acme
expect acme 'products.time[1 or "Rockets"]'
expect acme 'products.queuetime[2 or "Giant_Rubber_Bands"]'
total=$(awk -F' = ' '/products\.count\[/ { sum += $2 } END { print sum }' "$PCP_TMP_DIR/acme.txt")
if [ "$total" != 20 ]; then
    echo "expected 20 products in total, got $total" >&2
    exit 1
fi

# http: serve three GET requests on the loopback interface.
timeout 60 "$examples/http" 3 > "$PCP_TMP_DIR/http.log" &
http=$!
wait_for_line "$PCP_TMP_DIR/http.log" "Listening on"
ss -ltn 'sport = :6767' | tee "$PCP_TMP_DIR/http.ss"
if ! grep -q '127.0.0.1:6767' "$PCP_TMP_DIR/http.ss"; then
    echo "http example is not listening on 127.0.0.1 only" >&2
    exit 1
fi
for _ in 1 2 3; do
    curl -fsS http://127.0.0.1:6767/
done
wait "$http"
dump http
expect http 'http.get = 3'

# file: write a watched file twice.
watched="$PCP_TMP_DIR/watched.txt"
: > "$watched"
timeout 60 "$examples/file" "$watched" 2 > "$PCP_TMP_DIR/file.log" &
file=$!
wait_for_line "$PCP_TMP_DIR/file.log" "Watching"
echo one > "$watched"
wait_for_line "$PCP_TMP_DIR/file.log" "CLOSE_WRITE"
echo two > "$watched"
wait "$file"
dump file
expect file 'file.write_count = 2'

echo "All examples wrote valid MMV files."
