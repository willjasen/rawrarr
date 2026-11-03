# Movies cache first render

Issue: [#322](https://github.com/willjasen/rawrarr/issues/322)

## Change

On a refresh of the Movies view, restore the previously selected node from
`sessionStorage` and render its age-ID-gated movie text cache before waiting for
node health and route checks. Once node details load, refresh the library from
the node as before.

## Adoption and compatibility

No persistent data format, database schema, encrypted configuration, or node
protocol changes. Existing browser cache entries and session selection are
read as-is; users without a saved age ID or cached library continue through the
existing unlock flow. No migration is required. Deployment is the normal
coordinator application update.

## Verification

Not run. This change was reviewed through the source diff only.
