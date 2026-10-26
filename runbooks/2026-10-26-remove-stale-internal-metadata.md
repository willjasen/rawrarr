# Remove stale internal metadata after clean bootstrap

Issue [#115](https://github.com/willjasen/rawrarr/issues/115).

## Objective and initial state

The internal Movies view reported 1.1 GB of movie metadata and 1.2 GB total after the Oct 25, 2026 clean bootstrap. The coordinator could not decrypt the browser's saved config for the new identity. The Oct 25 deployment record says MacBook `/shared` was retained during that bootstrap. Live inspection found 4,892 files (1.2 GB) in `~/Library/Application Support/Rawrarr/shared-data/metadata` on the internal MacBook. All files were last modified Oct 18–21, before the bootstrap; subtree sizes were Movies 1.1 GB, TV 84 MB, Library 4.8 MB, and Books/Music 8 KB each.

The MacBook was reachable through its existing Tailscale host name and SSH host verification succeeded. A fresh check found `fossil` healthy and using the named `/shared` volume created Oct 22; that active volume also held 4,348 files (1.2 GB) under `/shared/metadata`, all last modified Oct 22. The host `shared-data/metadata` directory is not an active Syncthing folder according to the host Syncthing configuration. Both trees predate the clean bootstrap and are separate copies.

## Scope and safeguards

Empty only the old host `shared-data/metadata` directory and the active node volume’s `/shared/metadata` directory on the internal MacBook. Preserve the host `config.aged`, `age-recipient`, `.stfolder`, and `logs` entries, the other `/shared` paths, `/data`, coordinator state, Tailscale state, and all other files. No host `shared-data/media` directory existed. No file contents, age identities, or credentials were read or recorded. This deletes the retained encrypted metadata and artwork; there is no backup recorded for these trees.

## Actions

- Confirmed the host Syncthing configuration has no Rawrarr folder entry, so the old host copy is not an active Syncthing folder.
- Emptied only the direct children of host `shared-data/metadata` and container `/shared/metadata`; retained both parent directories and all other paths.
- Left the existing `fossil` service running.

## Verification

- Host metadata file count is 0. The host `shared-data` root still contains `.stfolder`, `config.aged`, `age-recipient`, and `logs`.
- Active `/shared/metadata` file count and reported size are both 0; `fossil` remains running and healthy.
- The authenticated Movies page now reports **0 B** for Radarr metadata and **0 B** for all metadata. It still reports “Connecting to Radarr through the node direct route…” and does not show library statistics; the saved browser identity could not decrypt the saved config before cleanup.
- The TV page showed only its heading during read-back, so a TV-specific card was not visually verified. The shared active metadata tree is empty.
- An initial media-path check reported that host `shared-data/media` is absent; no media files were removed. No Rawrarr folder appears in host Syncthing's active folder configuration.

## Final state and remaining work

Both pre-bootstrap metadata copies are empty and their storage directories remain present. Movie and total metadata sizes read 0 B. The saved config/decryption and Radarr connection issue is outside this cleanup and still needs owner setup or separate troubleshooting.
