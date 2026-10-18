# Media `.ev` folder layout

## Objective and initial state

Rename the MacBook's node-local `media-replica` directory to `media.ev` and reserve category folders under `media/` for future selective sync, including a node that receives only movies into `media/movies.ev`. The old directory holds about 3.3 GiB of synchronized music under `music/`. The running MacBook Syncthing container mounted `media-replica/music` at `/media`; the MacBook apex mounted `media-replica` read-only at `/media`. No `media.ev` or `media/` directory existed at the start. See the earlier [storage media replication runbook](2026-10-18-storage-media-replication.md).

## Actions and verification

- Before the move, Syncthing's `rawrarr-media` folder was receive-only and idle with 117 local/global files, zero needed files/bytes, and zero errors.
- Claimed separate local deployment locks for `rawrarr-local-syncthing-1` and `rawrarr-local-macbook-apex`. Stopped only those containers, moved the existing directory in place to `media.ev`, and recreated the containers with the new host bind paths. No media was copied or deleted, and their state volumes were retained.
- Read the Mac mini's `/Volumes/silo/Media/data` top-level directory names over SSH. They were `music`, `comics`, `images`, `audiobooks`, `roms`, `books`, `tv`, `podcasts`, and `movies`. Created an empty `.ev` folder for each under the MacBook's `media/` directory. These are placeholders; no category-specific Syncthing folder or node was configured.
- Updated the data-domain and media-sharing guides. The compose variable remains explicit, with a clearer path prompt.
- After the move, Syncthing mounted `media.ev/music` and MacBook apex mounted `media.ev`; both reached healthy status. Syncthing again reported the receive-only folder idle with 117 local/global files, zero needed files/bytes, and zero errors. Tailscale Serve JSON retained HTTPS 443 `/` to `http://127.0.0.1:8080`, `/direct-api` to `http://127.0.0.1:8082`, and TCP 22000 to Syncthing. The tailnet `/health` returned the Roarlay health page.

## Recovery and final state

If the new bind fails, stop the two affected containers under their deployment locks, move `media.ev` back to `media-replica`, and recreate them with the old bind paths. Do not delete the copied music or Syncthing state volume. The category placeholders can remain empty until a separately approved sync design uses them.

The active music replica remains under `media.ev/music`; the category placeholders under `media/` are empty and are not mounted into the running containers. A movies-only node, its authorization, and its Syncthing folder remain planned.
