# Share favicon across public Rawrarr sites

Issue: [#522](https://github.com/willjasen/rawrarr/issues/522)

## Objective and initial state

Use the same dinosaur icon for the static landing page at `rawrarr.net` and the
public app at `roarlay.rawrarr.net`, including SVG and ICO routes. The app's
unauthenticated favicon routes returned HTTP 200 with `image/svg+xml` and
`image/x-icon`; the landing page returned HTTP 404 for both paths. Its HTML
instead referenced a separate horizontal PNG logo. The site is deployed from
`website/` through Netlify.

Related history: [coordinator favicon fix](2026-11-03-coordinator-favicon-fix.md)
and [landing page deployment](2026-11-02-deploy-rawrarr-netlify-landing-page.md).

## Change and adoption

Copy the app's SVG to `website/assets/rawrarr-favicon.svg` and ICO to
`website/favicon.ico`, then advertise those assets on the landing page with
versioned URLs. Netlify serves both with explicit image MIME types and
revalidation (`max-age=0, must-revalidate`) so newly published artwork is
rechecked promptly. Netlify's immutable deployment model serves the matching
HTML and assets together; increment the query version if the artwork changes
again. No persistent data, schema, encryption, or protocol changes are involved,
so no migration is needed. Adoption requires publishing the updated `website/`
from `main` to Netlify; no coordinator deployment is required.

## Verification and status

- `node --test scripts/tests/test_rawrarr_favicon.mjs` passed in the isolated Chromium test image (4 tests). `node --test scripts/tests/test_netlify_ignore_build.mjs` passed (1 test). `git diff --check` passed.
- Live landing page and app assets were checked before implementation. The app
  serves the expected SVG and ICO. The landing page's corresponding paths are
  currently 404.
- Production recheck after Netlify publication: pending; this checkout has no Netlify CLI or site deployment connection. The current landing-page favicon URLs returned HTTP 404 before the change; the public app routes returned HTTP 200 with their image MIME types.

## Rollback

Revert the landing-page HTML, asset copies, Netlify headers, and test together,
then publish the prior Netlify deployment. This does not affect the app's
coordinator favicon routes.
