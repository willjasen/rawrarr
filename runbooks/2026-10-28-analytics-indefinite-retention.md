# Analytics indefinite retention decision

Issue: [#255](https://github.com/willjasen/rawrarr/issues/255)

Will clarified that analytics should be retained indefinitely. The temporary 90-day purge implementation was reverted before it was deployed to either analytics database. The existing Umami databases and coordinator services were not modified by the purge follow-up. Self-hosted Umami's default is to retain records indefinitely, and the deployment has no cleanup job.

No Rawrarr data migration is required. The correction changes only analytics documentation and removes the un-deployed purge service and SQL. Existing Umami volumes remain intact. As of 2026-10-28, both the internal and public do3 coordinator deployments are live with independent Umami websites and databases. Each coordinator was healthy, its tracker script loaded, and browser pageviews were present in the corresponding database. Umami retains the records indefinitely; no cleanup job is deployed. Dashboard listeners remain loopback-only.

Verification: `node --test scripts/tests/test_rawrarr_analytics.mjs` and `git diff --check`.
