# Tibia Client Editor

Desktop GUI for patching Tibia 11+ clients — a Rust + Tauri 2 reimplementation of [opentibiabr/client-editor](https://github.com/opentibiabr/client-editor).

## Credits

This project is based on [opentibiabr/client-editor](https://github.com/opentibiabr/client-editor). The RSA keys, binary patch flow, BattlEye signature handling, client-check diagnostics, config format, repack tooling, and appearances editing logic are ported from that repository.

Upstream reference files used directly:

- [`tibia_rsa.key`](https://github.com/opentibiabr/client-editor/blob/main/tibia_rsa.key)
- [`otserv_rsa.key`](https://github.com/opentibiabr/client-editor/blob/main/otserv_rsa.key)
- [`config.toml.dist`](https://github.com/opentibiabr/client-editor/blob/main/config.toml.dist)

Licensed under the GNU General Public License v3.0 — see [LICENSE](./LICENSE).

## Stack

- **Backend:** Rust (Tauri 2) — binary patching, BattlEye signatures, PE diagnosis, repack, appearances
- **Frontend:** Next.js 16, React 19, Tailwind CSS v4 (Bun)
- **UI:** [COSS UI](https://coss.com/ui/docs) (via `shadcn` + `@coss/style`)

## Features

| Tab | Description |
|-----|-------------|
| **Patch** | Replace embedded URLs, sync `config.ini`, apply RSA key and BattlEye patches |
| **Diagnose** | Inspect client-check compatibility without modifying files |
| **Repack** | Repack client folders for [slender-launcher](https://github.com/luan/slender-launcher) |
| **Appearances** | Edit item flags in `appearances.dat` via `[[edit]]` config blocks |
| **Win → Mac** | Convert Windows `assets.json` paths to macOS `Contents/Resources/` layout |

## Development

```bash
git clone https://github.com/orafal-dev/tibia-client-editor.git
cd tibia-client-editor
bun install
bun run tauri:dev
```

## Production build

```bash
bun run tauri:build
```

## Auto-updater and releases

The app uses the [Tauri updater plugin](https://v2.tauri.app/plugin/updater/) and checks GitHub releases on startup for a newer Windows build.

### Release a new version

1. Bump `version` in `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml`.
2. Commit the version bump.
3. Create and push a tag (must match the app version):

```bash
git tag v0.1.0
git push origin v0.1.0
```

GitHub Actions (`.github/workflows/release.yml`) builds on each `v*` tag push and publishes a GitHub release with:

| Asset | Purpose |
| --- | --- |
| `Tibia-Client-Editor_<version>_x64-setup.exe` | Windows NSIS installer for fresh installs |
| `Tibia-Client-Editor_<version>_x64-setup.exe.sig` | Updater signature |
| `latest.json` | Auto-update metadata for installed apps |

### One-time CI setup

Add the updater signing private key as a repository secret (generate locally with `bun run tauri signer generate -w src-tauri/.tauri/tibia-client-editor.key` if needed):

```bash
gh secret set TAURI_SIGNING_PRIVATE_KEY < src-tauri/.tauri/tibia-client-editor.key
```

If the key has a password, also set `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. The matching public key is already configured in `src-tauri/tauri.conf.json`.

## RSA keys

Same behavior as [opentibiabr/client-editor](https://github.com/opentibiabr/client-editor):

1. `tibia_rsa.key` and `otserv_rsa.key` are shipped in this repository (copied from upstream).
2. On patch, the tool reads those files from the **current working directory** first.
3. If not found there, it checks beside the executable.
4. Otherwise it uses the bundled keys compiled into the application.

To override the defaults, place custom key files in the working directory or next to the executable before patching.

## Config examples

See `examples/config.toml.dist` and `examples/local.toml`.
