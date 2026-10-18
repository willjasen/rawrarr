# Complete Radarr and Sonarr text metadata

## Objective

Persist the full text metadata exposed by Radarr and Sonarr in separate age-encrypted, HMAC-named JSON records under the shared metadata folder. Keep this provider data separate from storage-node file observations.

## Initial state

The metadata workers already backfilled movie and series text plus artwork to `/shared/metadata/movies/{text,artwork}` and `/shared/metadata/tv/{text,artwork}`. Text records included only selected fields and were written only when missing. Storage-node sidecars are separate under `/shared/metadata/library/text`.

## Actions

- Expand versioned Radarr movie and Sonarr series records to preserve provider text fields, including genre, studio/network, language, ratings, overview, status, and quality metadata. Include Sonarr episode records with series metadata.
- Strip local filesystem paths and byte-size fields from Arr payloads before sharing; keep storage-node sidecars independent.
- Refresh records from the six-hour metadata-node workers and encrypt them to the existing public age recipient using the existing HMAC filenames.
- Update the data-domain and product-design documentation to describe this boundary.

## Verification

- `docker compose -p rawrarr-local -f docker-compose.yml -f docker-compose.coordinator.yml -f docker-compose.tailscale-macbook.yml build metadata` completed successfully. The release build emitted one pre-existing dead-code warning for `read_shared_recipient`.
- Recreated only `rawrarr-local-metadata-1` using the recorded Compose project and overlays. The container reports healthy and its internal `http://127.0.0.1:8081/health` endpoint returned HTTP 200.
- The shared folder contained 2,172 files matching `*.json.aged` under the Radarr and Sonarr text directories at deployment time. Filenames and decrypted contents were not inspected. This count does not confirm the next refresh completed.
- Backfill status endpoints were not readable unauthenticated: the direct Sonarr status returned 401 and the direct Radarr status path returned 404. Worker completion after the deployment therefore remains unverified.
- `git diff --check` passed for the task source and documentation files. No test suite was run.

Never include decrypted production payloads, credentials, or private age identities.

## Recovery

Revert this task's source and documentation changes. Existing encrypted records remain intact; an earlier binary will ignore schema version 2 until the updated metadata service is deployed again.
