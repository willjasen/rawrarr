# Analytics indefinite retention decision

Issue: [#255](https://github.com/willjasen/rawrarr/issues/255)

Will clarified that analytics should be retained indefinitely. The temporary 90-day purge implementation was reverted before it was deployed to either analytics database. The existing Umami databases and coordinator services were not modified by the purge follow-up. Self-hosted Umami's default is to retain records indefinitely, and the deployment has no cleanup job.

No Rawrarr data migration is required. The correction changes only analytics documentation and removes the un-deployed purge service and SQL. Existing Umami volumes remain intact. Analytics tracking remains disabled until each environment has an independent website/site ID configured.

Verification: `node --test scripts/tests/test_rawrarr_analytics.mjs` and `git diff --check`.
