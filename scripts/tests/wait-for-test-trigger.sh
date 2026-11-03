#!/bin/sh
set -eu
while [ ! -f /tmp/rawrarr-test-ready ]; do
    sleep 1
done
exec sh /workspace/scripts/tests/run-tests-in-container.sh
