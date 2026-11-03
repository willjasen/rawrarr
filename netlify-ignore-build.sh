#!/bin/sh

# Netlify skips a Git-connected build on exit 0 and continues it on nonzero.
# Missing refs or Git errors fail open so deploys are not accidentally blocked.
if [ -z "${CACHED_COMMIT_REF:-}" ] || [ -z "${COMMIT_REF:-}" ]; then
  exit 1
fi

if ! git cat-file -e "${CACHED_COMMIT_REF}^{commit}" 2>/dev/null || ! git cat-file -e "${COMMIT_REF}^{commit}" 2>/dev/null; then
  exit 1
fi

git diff --quiet "$CACHED_COMMIT_REF" "$COMMIT_REF" -- website netlify.toml netlify-ignore-build.sh
diff_status=$?

if [ "$diff_status" -eq 0 ]; then
  exit 0
fi

exit 1
