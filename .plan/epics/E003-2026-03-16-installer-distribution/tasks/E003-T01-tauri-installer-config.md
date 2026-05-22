---
id: E003-T01
epic: E003
status: completed
original_id: T001
title: Configure Windows Installer (NSIS, MSI, Signing Scaffold)
---
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

### 3. Code Signing Scaffold

Create `scripts/sign-installer.sh` template.

### 4. Auto-Update Scaffold (Comment in tauri.conf.json)

### 5. CI/CD Integration (GitHub Actions)

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

## Blocks

- T002: Build report (needs artifacts to exist)
- T003: Memory profiling (needs stable Tauri config)
- T004: User docs (references installer features)
