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

1. **Update the changelog.** In `CHANGELOG.md`, rename `## [Unreleased]` to
   `## [x.y.z] - YYYY-MM-DD` and start a fresh empty `## [Unreleased]` above it.
   Keep entries short and user-facing (Added / Changed / Fixed). The workflow
   pulls this section into the GitHub release notes, and the app shows it under
   "What's new".
2. Bump the version in **both** `package.json` and `src-tauri/tauri.conf.json`
   (e.g. `0.2.0`). They should always match — the status-bar version reads
   `package.json`; the updater compares against `tauri.conf.json`.
3. Commit, then tag and push:
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```
4. The **Release** GitHub Action builds + signs the installer and a signed
   updater bundle, and creates a **draft** release with `latest.json`.
5. Open the release on GitHub and **Publish** it.

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
Bletchley). On first launch the app creates its SQLite database; timecodes are
then pulled from Replicon via Settings → Sync timecodes.


## Testing updates locally (no releases)
The install step needs real `tauri build` artifacts — it can't run under
`tauri dev` (there's no installer to hand off to). To exercise the whole
check → download → install → relaunch loop without publishing releases:

1. Put your signing key in the environment (same as a real build).
2. Build the **"from" version** (the one that must contain the fix you're
   testing) pointed at a local server, and install it:
   ```powershell
   npm run tauri build -- --config src-tauri/tauri.update-test.json
   ```
   `tauri.update-test.json` (gitignored) merges a `http://localhost:8080`
   endpoint + the insecure-transport flag over the real config, so the
   localhost endpoint never ships. Run the produced `-setup.exe` to install.
3. Bump the version, build the **"to" version** normally (`npm run tauri build`).
   In `src-tauri/target/release/bundle/nsis/` you'll get
   `Bletchley_<new>_x64-setup.nsis.zip` and its `.sig`.
4. Make an `update-test/` folder (gitignored), copy the `.nsis.zip` into it,
   and add `latest.json`:
   ```json
   {
     "version": "<new version>",
     "notes": "local test",
     "pub_date": "2026-01-01T00:00:00Z",
     "platforms": {
       "windows-x86_64": {
         "signature": "<contents of the .nsis.zip.sig file>",
         "url": "http://localhost:8080/Bletchley_<new>_x64-setup.nsis.zip"
       }
     }
   }
   ```
5. Serve it: from `update-test/`, run `python -m http.server 8080`.
6. Launch the installed "from" version. The update pill appears; clicking it
   downloads from localhost, installs, and relaunches into the new version.
   On Windows the app force-exits during install (installer limitation).

Real builds (without `--config`) keep the GitHub HTTPS endpoint.
