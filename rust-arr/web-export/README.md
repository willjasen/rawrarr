# Browser metadata export bundle

The browser proof of concept uses the pinned `age-encryption` TypeScript implementation for age decryption, including hybrid post-quantum identities, and `fflate` for an in-browser ZIP archive. Source is in `metadata-export.js`; `npm run build` produces the static bundle at `../src/assets/metadata-export.js`, which Rust embeds in the service. Dependencies are pinned by `package-lock.json`.

After changing the source or dependencies, run:

```sh
npm ci
npm run build
```

The mini-export browser test also uses the age package and `fflate` from this dependency tree. Its ciphertext fixtures are generated ephemerally by the installed `age` CLI.
