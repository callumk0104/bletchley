# Releasing & auto-update

Bletchley ships as a per-user Windows installer (NSIS `.exe`) and updates
itself in-app via the Tauri updater, using **GitHub Releases** as the source.

## One-time setup

### 1. Generate a signing key
Updates must be signed; the app only installs updates that match the public key
baked into the build.

```bash
npm run tauri signer generate -- -w "$HOME/.tauri/bletchley.key"
```

This prints a **public key** and writes a **private key** file (with the
password you choose). Then:

- Paste the **public key** into `src-tauri/tauri.conf.json` →
  `plugins.updater.pubkey` (replacing `PASTE_YOUR_TAURI_PUBLIC_KEY_HERE`).
- Keep the private key secret. Add two **GitHub repository secrets**
  (Settings → Secrets and variables → Actions):
  - `TAURI_SIGNING_PRIVATE_KEY` — contents of the private key file.
  - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — the password you set.

### 2. Point the updater at your repo
In `src-tauri/tauri.conf.json` → `plugins.updater.endpoints`, confirm the repo
matches yours:

```
https://github.com/<owner>/<repo>/releases/latest/download/latest.json
```

(Currently set to `callumk0104/bletchley` — change if that's not right.)

## Cutting a release

1. Bump the version in **both** `package.json` and `src-tauri/tauri.conf.json`
   (e.g. `0.2.0`). They should always match — the status-bar version reads
   `package.json`; the updater compares against `tauri.conf.json`.
2. Commit, then tag and push:
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```
3. The **Release** GitHub Action builds + signs the installer and a signed
   updater bundle, and creates a **draft** release with `latest.json`.
4. Open the release on GitHub and **Publish** it.

That's it. Running apps check `latest.json` on startup; when a newer signed
version exists, the status bar shows an "Update to x.y.z" pill that downloads,
installs, and relaunches on click.

## Building an installer locally (optional)
Requires the signing env vars set (because `createUpdaterArtifacts` is on):

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content "$HOME/.tauri/bletchley.key" -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "your-password"
npm run tauri build
```

Output: `src-tauri/target/release/bundle/nsis/Bletchley_<version>_x64-setup.exe`
(plus the signed `.nsis.zip` + `.sig` used for auto-update).

## How "new user setup" works
The installer drops the app + a Start-Menu shortcut into the user profile (no
admin needed) and registers the app identity (so notifications show as
Bletchley). On first launch the app creates its SQLite database and seeds a
starter set of timecodes automatically — nothing else to configure.
