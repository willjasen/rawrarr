#!/bin/sh
set +e

echo "rawrarr isolated test runner"
echo "Rust: $(rustc --version)"
echo "Node: $(node --version)"
test_started_epoch=$(date +%s)
test_started_at=$(date '+%Y-%m-%d %H:%M:%S %z')

node_log="/tmp/rawrarr-node-tests.$$.log"
playwright_log="/tmp/rawrarr-playwright-tests.$$.log"
rust_log="/tmp/rawrarr-rust-tests.$$.log"
browser_test=/workspace/scripts/tests/test_rawrarr_age_setup_browser.mjs
trap 'rm -f "$node_log" "$playwright_log" "$rust_log"' EXIT

section_start() {
    printf '\n==================== %s ====================\n' "$1"
}

section_end() {
    printf '==================== END %s (status=%s) ====================\n' "$1" "$2"
}

section_start "NODE.JS TESTS"
node_files=""
for test_file in /workspace/scripts/tests/*.mjs; do
    if [ "$test_file" = "$browser_test" ]; then
        continue
    fi
    node_files="$node_files $test_file"
done
node --test $node_files >"$node_log" 2>&1
node_status=$?
cat "$node_log"
section_end "NODE.JS TESTS" "$node_status"

section_start "PLAYWRIGHT BROWSER TEST"
node --test "$browser_test" >"$playwright_log" 2>&1
playwright_status=$?
cat "$playwright_log"
section_end "PLAYWRIGHT BROWSER TEST" "$playwright_status"

section_start "RUST UNIT TESTS"
cargo clean --manifest-path /workspace/rust-arr/Cargo.toml -p rust-arr
cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml >"$rust_log" 2>&1
rust_status=$?
cat "$rust_log"
section_end "RUST UNIT TESTS" "$rust_status"

section_start "PYTHON INTEGRATION TESTS"
echo "SKIP: Python integration tests need disposable PostgreSQL, age, or Syncthing fixtures; they are not run by this unit-test container."
echo "SKIP: OrbStack peer-container sync is run separately by test_rawrarr_syncthing_orbstack.py on the host Docker context."
section_end "PYTHON INTEGRATION TESTS" "skipped"

printf 'TEST RESULTS playwright=%s node=%s rust=%s\n' "$playwright_status" "$node_status" "$rust_status"
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
    failures=$(grep -E '(^not ok [0-9]+|^test .* \.\.\. FAILED|^error:|^error\[)' "$log_file" || true)
    if [ -n "$failures" ]; then
        printf '%s\n' "$failures" | sed 's/^/  /'
    else
        echo "  No individual failing test was reported; see the suite log above."
    fi
}
print_result "Node.js" "$node_status" "$node_log"
print_result "Playwright browser" "$playwright_status" "$playwright_log"
print_result "Rust" "$rust_status" "$rust_log"
test_stopped_epoch=$(date +%s)
test_stopped_at=$(date '+%Y-%m-%d %H:%M:%S %z')
test_duration=$((test_stopped_epoch - test_started_epoch))
printf 'Test start: %s\n' "$test_started_at"
printf 'Test stop:  %s\n' "$test_stopped_at"
printf 'Test run:   %s seconds\n' "$test_duration"
if [ "$node_status" -eq 0 ] && [ "$playwright_status" -eq 0 ] && [ "$rust_status" -eq 0 ]; then
    echo "ALL CONTAINER TESTS PASSED"
    echo "================================================================"
    exit 0
fi
echo "CONTAINER TESTS FAILED"
echo "================================================================"
exit 1
