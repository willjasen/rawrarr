# Movies initial load and imported age ID

Issue: [#369](https://github.com/willjasen/rawrarr/issues/369)

## Findings and change

Movies initialization waited for optional Syncthing, peer, and route diagnostics
before selecting a node. A restored node selection was assigned details without
running its normal opening flow. Select and open the available node as soon as
browser route checks finish, preserving the lowest-latency default selection
without waiting for optional diagnostics. Later diagnostics update labels
without resetting the library.

The shared JSON request helper cleared the browser age identity on every HTTP
401, including rejected node proxy requests. Failed requests now report their
authentication error without deleting the imported identity. Explicit sign-out
and clear-ID actions retain their existing clearing behavior.

Hash navigation now runs the same view-loading flow as sidebar navigation.
Pages restored from the browser back/forward cache reload the saved identity
after the existing page-hide memory cleanup and refresh the active view.

## Adoption and compatibility

This is coordinator browser behavior only. Existing age IDs, library caches,
node selection, encrypted configuration, and protocols keep their current
formats. No migration or subscriber update is required. Existing deployments
adopt through a coordinator application update from published GitHub main;
an open browser must reload its UI assets. No deployment is part of this task.

## Verification

- The four focused age-ID/Movies Chromium cases pass in the offline OrbStack
  clone, including desktop/mobile geometry, automatic initial loading while
  diagnostics remain pending, hash navigation, and simulated persisted
  page-hide/page-show restoration. A restored page also removes the old
  library when the saved identity has been cleared.
- Related workspace, direct-route, and text-cache checks pass (44 focused
  checks including the four browser cases).
- The final full isolated runner passed all 151 Node checks, 116 Rust tests,
  and all four Movies/age-ID browser cases.
- Two other browser failures reproduce with the unchanged HEAD workspace
  script: the direct Radarr/Sonarr settings case waits for an unconfigured
  Radarr note, and the signing-topology case cannot see its desktop section
  title. They are outside this task's changes.
- The final full browser rerun fails only those two baseline cases. Python
  integration and full multi-node environment tests were not run. No live
  deployment was performed.
