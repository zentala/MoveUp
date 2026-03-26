# Installer & Distribution

## Build Command

```bash
pnpm tauri:build
```

1. Runs `pnpm test:all` (unit + Rust tests must pass)
2. Builds React frontend (optimized, code-split)
3. Compiles Rust backend with LTO
4. Generates Windows installer (NSIS)
5. Output: `src-tauri/target/release/bundle/`

## Size Targets

- **Installer:** 60–70 MB | **Peak memory:** < 250 MB | **Stable memory:** < 200 MB
- Check: `pnpm build:report` → `.build-sizes.json`
- Profile: `pnpm test:perf` → `.perf-baseline.json`

## Icons

Must exist in `src-tauri/icons/`: `32x32.png`, `128x128.png`, `icon.ico`

## Code Signing (Scaffolded)

Not yet active. `scripts/sign-installer.sh` is a placeholder.
Needs: certificate (.pfx) from DigiCert/GlobalSign (~$200-500/yr), GitHub Secrets.
Activate when releasing publicly.

## Auto-Update (Scaffolded)

Disabled in `tauri.conf.json`. Activate after code signing is set up.
Checks GitHub Releases every 24h for new versions.

## Pre-Release Checklist

```bash
pnpm test:all && pnpm test:perf && cat .perf-baseline.json && cat .build-sizes.json
```
Then: `git tag vX.Y.Z` → `git push origin vX.Y.Z`
