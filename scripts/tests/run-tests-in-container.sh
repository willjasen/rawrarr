#!/bin/sh
set +e

echo "rawrarr isolated test runner"
echo "Rust: $(rustc --version)"
echo "Node: $(node --version)"

echo "=== Node.js tests ==="
node --test /workspace/scripts/tests/*.mjs
node_status=$?

echo "=== Rust unit tests ==="
cargo clean --manifest-path /workspace/rust-arr/Cargo.toml -p rust-arr
cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml
rust_status=$?

echo "=== Python integration tests ==="
echo "SKIP: Python integration tests need disposable PostgreSQL, age, or Syncthing fixtures; they are not run by this unit-test container."
echo "SKIP: OrbStack peer-container sync is run separately by test_rawrarr_syncthing_orbstack.py on the host Docker context."

printf 'TEST RESULTS node=%s rust=%s\n' "$node_status" "$rust_status"
if [ "$node_status" -eq 0 ] && [ "$rust_status" -eq 0 ]; then
    echo "ALL CONTAINER TESTS PASSED"
    exit 0
fi
echo "CONTAINER TESTS FAILED"
exit 1
