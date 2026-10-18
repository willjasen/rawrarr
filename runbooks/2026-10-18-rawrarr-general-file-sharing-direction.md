# Rawrarr as a device-to-device file-sharing system

## Objective and initial state

Capture the user's product direction that Rawrarr extends the broader Synkpieces idea: sharing files between devices is the core, while Radarr and other media workflows are one important use case. Current implementation and documentation are still focused on media automation and encrypted shared application data.

## Actions

- Updated `docs/product-direction.md` to describe general file sharing as a future product area and preserve media automation as the first focused use case.
- Updated `docs/DESIGN.md` so future navigation and library design can handle ordinary shared files and folders without requiring media metadata.
- Left implementation scope unchanged; general-purpose file/folder sharing is a direction to revisit, not a feature claimed as implemented.

## Verification and remaining work

Review the documentation diff and run `git diff --check`. Future design work should define folder sharing, trust, encryption, and synchronization behavior for non-media data before implementation.
