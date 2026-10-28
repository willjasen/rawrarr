# Analytics retention and rollout

Issue: [#255](https://github.com/willjasen/rawrarr/issues/255)

## Change

Enforce a 90-day maximum for Umami visitor and event records in the self-hosted internal and public coordinator deployments. The existing Rawrarr data, schema, subscriber protocol, and `/shared` formats are unchanged. This is an analytics-database-only retention policy; it has no migration of existing Rawrarr state. On first start, Umami data older than 90 days is purged.

`umami-retention` uses the PostgreSQL 17 client image and runs a transaction against the Umami 3.4.1 schema at startup and every 24 hours. It deletes child event/session records before parent sessions, and keeps sessions referenced by any remaining event. A failed transaction rolls back and is logged; the container retries on its next daily cycle or restart. The SQL must be reviewed against the Umami schema before upgrading the pinned Umami release. The job is repeatable.

## Adoption and recovery

Add the updated `docker-compose.analytics.yml` and `deploy/analytics/retention.sql`, then recreate only `umami-retention` in each coordinator Compose project. Existing Umami volumes remain intact. The job immediately purges records past cutoff, so take an Umami database backup before adoption if records older than 90 days may need recovery. Rollback by reverting the overlay and stopping/removing `umami-retention`; this stops future purges but cannot recover deleted rows. Restore a pre-purge database backup to recover them. To pause retention before a planned maintenance window, stop `umami-retention` and start it again afterwards.

## Verification

- Run `node --test scripts/tests/test_rawrarr_analytics.mjs` and `git diff --check`.
- Validate the overlay against both project Compose files and confirm `umami-retention` is running and its logs show a successful transaction.
- Confirm old test rows in every visitor/event table are removed and rows at or newer than the cutoff remain; do this only in a disposable Umami database, never against production analytics data.
- Verify each coordinator's `/analytics/script.js`, event POST and separate Umami dashboard after site IDs are configured.

## Deployment record

Pending publication and deployment of this retention follow-up. Internal and public Umami stacks have already been started separately with empty site IDs; collection remains disabled until this purge is deployed and each environment's independent website is configured.
