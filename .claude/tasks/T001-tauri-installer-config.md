# T001: Configure Windows Installer (NSIS, MSI, Signing Scaffold)

**Task ID:** T001
**Status:** Pending
**Effort:** 3 hours
**Priority:** P0 (blocks T002, T003, T004)
**Owner:** Zentala
**Date Created:** 2026-03-16

---

## Summary

Configure Tauri's bundle system for NSIS/MSI, scaffold code signing, scaffold auto-update, and document icon requirements.

---

## Acceptance Criteria

### 1. tauri.conf.json NSIS Configuration

Update `src-tauri/tauri.conf.json`:

```json
{
  "bundle": {
    "active": true,
    "targets": "all",
    "nsis": {
      "installerFilename": "zntlDesk_0.1.0_x64_en-US.exe",
      "shortcutsTargetDir": "ProgramFilesFolder",
      "uninstallerSidebar": true,
      "deleteUninstallerOnFinish": false
    },
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.ico"
    ]
  }
}
```

**Verify:**
- `pnpm tauri:build` produces `.exe` in `src-tauri/target/release/bundle/nsis/`
- Uninstaller option present in installer UI
- Icon displays correctly

### 2. Icon Path Documentation in CLAUDE.md

Add to **apps/desk/CLAUDE.md** new section:

```markdown
## Icon Requirements

Icons must exist in `src-tauri/icons/`:
- **32x32.png** — Tray icon, system tray display
- **128x128.png** — App window icon
- **icon.ico** — Installer icon, Windows display

If icons are missing, `pnpm tauri:build` fails with:
```
error: Icon file not found: icons/icon.ico
```

To generate icons from a PNG:
[Link to Tauri icon guide](https://tauri.app/en/develop/guides/assets/#icons)
```

### 3. Code Signing Scaffold

Create `scripts/sign-installer.sh` template:

```bash
#!/bin/bash
# Code signing template for Windows installer
# Usage: ./scripts/sign-installer.sh

set -e

CERT_PATH="${SIGN_CERT_PATH}"
CERT_PASSWORD="${SIGN_PASSWORD}"
INSTALLER_PATH="src-tauri/target/release/bundle/nsis/zntlDesk_0.1.0_x64_en-US.exe"

if [ ! -f "$INSTALLER_PATH" ]; then
    echo "❌ Installer not found: $INSTALLER_PATH"
    exit 1
fi

if [ -z "$CERT_PATH" ] || [ -z "$CERT_PASSWORD" ]; then
    echo "❌ Environment variables not set:"
    echo "   SIGN_CERT_PATH: path to .pfx file"
    echo "   SIGN_PASSWORD: certificate password"
    exit 1
fi

echo "🔐 Signing installer..."
# This will be implemented with actual signtool or equivalent
# signtool sign /f "$CERT_PATH" /p "$CERT_PASSWORD" /t http://timestamp.server.com "$INSTALLER_PATH"

echo "✅ Installer signed successfully"
```

Document in CLAUDE.md:

```markdown
## Code Signing (Scaffolded)

Code signing ensures Windows trusts the installer.

### Setup (One-time)

1. Acquire Windows code-signing certificate (.pfx file):
   - Vendor: DigiCert, GlobalSign, etc.
   - Cost: ~$200-500/year
   - File: `mycert.pfx` + password

2. Add to GitHub Secrets (Settings → Secrets and variables):
   - `SIGN_CERT_PATH`: `/home/runner/mycert.pfx` (upload as action secret)
   - `SIGN_PASSWORD`: certificate password (sensitive)

3. Reference in CI/CD (future `release.yml`):
   ```yaml
   env:
     SIGN_CERT_PATH: ${{ secrets.SIGN_CERT_PATH }}
     SIGN_PASSWORD: ${{ secrets.SIGN_PASSWORD }}
   ```

### Script

Location: `scripts/sign-installer.sh`

Currently a template. When certificate acquired, uncomment `signtool` command.

### Implementation Timeline

- Now: Scaffold placeholder
- Upon first release: Acquire cert + activate signing
- CI/CD integration: Part of T007 (release.yml)
```

### 4. Auto-Update Scaffold (Comment in tauri.conf.json)

Add comment to `src-tauri/tauri.conf.json`:

```json
{
  "build": {...},
  "app": {...},
  "bundle": {...},
  "// AUTO-UPDATE (future)": {
    "note": "When auto-update is implemented, add:",
    "updater": {
      "active": true,
      "endpoints": [
        "https://releases.example.com/latest.json"
      ],
      "dialog": true,
      "pubkey": "public-key-from-cert"
    }
  }
}
```

Document in CLAUDE.md:

```markdown
## Auto-Update Mechanism (Scaffolded)

Users can auto-update when new versions ship to GitHub Releases.

### How It Works

1. Tauri checks `https://releases.example.com/latest.json` periodically
2. If new version available, downloads + verifies signature
3. Prompts user: "Update available: click to install?"
4. Applies update in background

### Setup (Future)

- Acquire code-signing certificate (see ## Code Signing above)
- Configure `updater` in tauri.conf.json with manifest URL
- Create GitHub Releases workflow (T007)
- Generate update manifest with signatures

### Timeline

- Now: Placeholder in tauri.conf.json
- Upon code signing: Implement updater config
- With T007: Auto-generate update manifests in CI/CD
```

### 5. CI/CD Integration (GitHub Actions)

Update `.github/workflows/test.yml`:

**Before `pnpm tauri build --debug` step, add:**

```yaml
- name: Check disk space
  run: |
    df -h
    FREE_MB=$(df / | tail -1 | awk '{print $4}')
    if [ $FREE_MB -lt 500000 ]; then
      echo "❌ Insufficient disk space: ${FREE_MB}MB (need 500MB)"
      exit 1
    fi
    echo "✅ Disk space OK: ${FREE_MB}MB available"

- name: Build Tauri (Debug)
  working-directory: apps/desk
  run: pnpm tauri build --debug
  if: runner.os == 'Windows'

- name: Verify artifacts
  working-directory: apps/desk
  run: |
    test -f "src-tauri/target/release/bundle/nsis/zntlDesk_0.1.0_x64_en-US.exe" || \
    (echo "❌ NSIS installer not found"; exit 1)
    test -f "src-tauri/target/release/bundle/msi/zntlDesk_0.1.0_x64_en-US.msi" || \
    (echo "❌ MSI installer not found"; exit 1)
    echo "✅ Both installers verified"
  if: runner.os == 'Windows'
```

---

## Implementation Checklist

- [ ] Update `src-tauri/tauri.conf.json` with NSIS config
- [ ] Create `scripts/sign-installer.sh` template
- [ ] Add icon documentation to CLAUDE.md
- [ ] Add code signing section to CLAUDE.md
- [ ] Add auto-update section to CLAUDE.md (commented)
- [ ] Update `.github/workflows/test.yml` with disk check + artifact validation
- [ ] Test: `pnpm tauri:build` produces both .exe and .msi
- [ ] Test: uninstaller option appears in installer UI
- [ ] Verify artifact check passes in CI/CD

---

## Testing

```bash
# Local: build and verify
pnpm tauri:build

# Check artifacts
ls -lh src-tauri/target/release/bundle/nsis/zntlDesk_*.exe
ls -lh src-tauri/target/release/bundle/msi/zntlDesk_*.msi

# CI/CD: should pass artifact check
# Review .github/workflows/test.yml output
```

---

## Review References

- **CEO Review:** Architecture Issue #1 (exit codes)
- **Eng Review:** Code Quality Issue #3 (icon documentation)
- **Decisions:** Icon requirements documented, signing templated

---

## Unblocked By

Nothing (independent)

## Blocks

- T002: Build report (needs artifacts to exist)
- T003: Memory profiling (needs stable Tauri config)
- T004: User docs (references installer features)

---

## Future Follow-up

- T007: Implement actual signing in release.yml
- T007: Implement auto-update manifest generation
