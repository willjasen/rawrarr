# Browser metadata export bundle

The browser proof of concept uses the pinned `age-encryption` TypeScript implementation for age decryption, including hybrid post-quantum identities, and `fflate` for an in-browser ZIP archive. Source is in `metadata-export.js`; `npm run build` produces the static bundle at `../src/assets/metadata-export.js`, which Rust embeds in the service. Dependencies are pinned by `package-lock.json`.

After changing the source or dependencies, run:

```sh
npm ci
npm run build
```

The mini-export browser test also uses the age package and `fflate` from this dependency tree. Its ciphertext fixtures are generated ephemerally by the installed `age` CLI.

`npm run build:subscriber-onboarding` bundles the browser-only age and encrypted signing-root setup at `/setup/node`. `subscriber-compose.js` builds a fossil or boneyard Compose file with an explicit internal Tailscale or public Headscale control-server mode. Subscribers do not receive the coordinator-only private GitHub UI updater token. Both modes use persistent node-local Tailscale state. At the owner's request, internal Tailscale mode embeds the entered one-time auth key in the downloaded Compose file; keep that file private and remove the key after the node joins. Headscale mode continues to use a separate local Compose secret file. The overlay provides node networking only: coordinator reachability, Rawrarr approval, and Syncthing `/shared` replication remain separate setup steps. Rebuild after changing either source module; the generated bundle is embedded in roarlay. The public release image and active signed-approval trust anchor are separate remaining work.
