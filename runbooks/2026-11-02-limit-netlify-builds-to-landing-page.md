# Limit Netlify builds to landing page changes

Issue: [#336](https://github.com/willjasen/rawrarr/issues/336)

## Scope

The Netlify config skips Git-connected builds unless the commit range changes `website/`, `netlify.toml`, or the ignore helper itself. The static site still publishes only `website/`. Requests to `http://rawrarr.net/` and any HTTP path on that hostname receive a forced permanent redirect to the same path over HTTPS. Other hostnames and HTTPS requests are unaffected.

## Compatibility and adoption

There is no persistent application state, data migration, node protocol, or deployment target change. Netlify adopts the behavior when the updated `netlify.toml` reaches its configured Git branch. The ignore command returns 0 to skip and nonzero to continue; missing refs and Git errors fail open so a build proceeds safely. Netlify documents that build-hook-triggered deploys are not canceled by an ignore command; this rule covers Git-connected builds. HTTP redirects are declared as forced domain-level rules because the site also serves static files at `/`.

## Verification

`test_netlify_ignore_build.mjs` creates a temporary Git history and checks backend-only changes are skipped while website, Netlify config, and helper changes trigger a build; it also checks missing refs fail open and that the TOML defines forced HTTP-to-HTTPS redirects for the apex and its paths. Run with `node --test scripts/tests/test_netlify_ignore_build.mjs`. Verify the deployed response with `curl -sSI http://rawrarr.net/` and confirm a `301` to `https://rawrarr.net/`; verify an HTTPS request still serves the landing page.
