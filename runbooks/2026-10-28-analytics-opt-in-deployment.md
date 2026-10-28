# Make coordinator analytics deployment opt-in

Issue: https://github.com/willjasen/rawrarr/issues/286

## Change

Keep the coordinator analytics overlay limited to optional environment settings, both empty by default. Move Umami and its dedicated PostgreSQL service into `docker-compose.analytics-services.yml`, which operators must invoke explicitly. This keeps routine coordinator deployments independent from Umami setup and its secrets.

No Rawrarr or Umami data, database schema, node protocol, or synchronized `/shared` data changes. No migration is required. Existing analytics containers, volumes, dashboard routes, and credentials remain as-is; this code task does not deploy to do3 or remove any service.

## Verification

- Run `node --test scripts/tests/test_rawrarr_analytics.mjs`.
- Confirm the merged coordinator configuration lists only its standard services and renders both analytics settings empty when they are not supplied. Validation may use `/dev/null` as the required secret file paths; do not start containers.
- Validate `docker-compose.analytics-services.yml` with dummy, non-secret values for its required interpolation variables. Do not start services during validation.

## Enable analytics later

Create the Umami credentials and private network in the protected host environment, then run the service Compose file explicitly:

```sh
docker compose -p rawrarr-internal-do3 --env-file /path/to/protected/analytics.env \
  -f docker-compose.analytics-services.yml up -d
```

Set `RAWARR_ANALYTICS_UPSTREAM=http://umami:3000` and the environment's website ID before recreating its coordinator. Keep public and internal databases, credentials, site IDs, and network settings separate.
