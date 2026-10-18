# Deploy the age ID refresh fix locally

## Objective and initial state

Apply the camel-case age identity request decoding fix to the persistent local Rawrarr service. The app was healthy at `127.0.0.1:7297`, and the `rawrarr-local` Compose project had separate persistent app data, shared data, and PostgreSQL storage.

## Actions

- Confirmed port 7297 was occupied only by the existing `rawrarr-local` app.
- Built from the current checkout and force-recreated only the `rawrarr` service, carrying forward the running container's Rawrarr environment values, protected password-file mount, shared-data bind, and required host port.
- Left PostgreSQL running and did not navigate or reload the user's browser windows.

## Verification

- The app is healthy and uses the rebuilt `rawrarr:local` image.
- `GET http://127.0.0.1:7297/health` returned HTTP 200 `ok`.
- The app retained its `rawrarr-local_rawrarr-data` volume, shared-data path, and `127.0.0.1:7297` binding; PostgreSQL remains healthy.
- Earlier in this task, the Docker test suite passed all Node.js, Chromium, and Rust tests, including the request-decoding regression.

## Recovery and final state

The local service now contains the age ID persistence fix. If an ID was entered before this deployment, import it once more and select **Keep this ID on this device**; the prior request bug prevented the selection from being saved.
