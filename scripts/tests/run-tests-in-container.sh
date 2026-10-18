#!/bin/sh
set +e

echo "rawrarr isolated test runner"
echo "Rust: $(rustc --version)"
echo "Node: $(node --version)"

node_log="/tmp/rawrarr-node-tests.$$.log"
rust_log="/tmp/rawrarr-rust-tests.$$.log"
trap 'rm -f "$node_log" "$rust_log"' EXIT

echo "=== Node.js tests ==="
node --test /workspace/scripts/tests/*.mjs >"$node_log" 2>&1
node_status=$?
cat "$node_log"

echo "=== Rust unit tests ==="
cargo clean --manifest-path /workspace/rust-arr/Cargo.toml -p rust-arr
cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml >"$rust_log" 2>&1
rust_status=$?
cat "$rust_log"

echo "=== Python integration tests ==="
echo "SKIP: Python integration tests need disposable PostgreSQL, age, or Syncthing fixtures; they are not run by this unit-test container."
echo "SKIP: OrbStack peer-container sync is run separately by test_rawrarr_syncthing_orbstack.py on the host Docker context."

printf 'TEST RESULTS node=%s rust=%s\n' "$node_status" "$rust_status"
echo
echo "================================================================"
echo "                    CONTAINER TEST SUMMARY"
echo "================================================================"
print_result() {
    suite=$1
    status=$2
    log_file=$3

    if [ "$status" -eq 0 ]; then
        echo "$suite: passed"
        return
    fi

    echo "$suite: failed (exit $status)"
    failures=$(grep -E '(^not ok [0-9]+|^test .* \\.\\.\\. FAILED|^error:|^error\[)' "$log_file" || true)
    if [ -n "$failures" ]; then
        printf '%s\n' "$failures" | sed 's/^/  /'
    else
        echo "  No individual failing test was reported; see the suite log above."
    fi
}
print_result "Node.js" "$node_status" "$node_log"
print_result "Rust" "$rust_status" "$rust_log"
if [ "$node_status" -eq 0 ] && [ "$rust_status" -eq 0 ]; then
    echo "ALL CONTAINER TESTS PASSED"
    echo "================================================================"
    exit 0
fi
echo "CONTAINER TESTS FAILED"
echo "================================================================"
exit 1
