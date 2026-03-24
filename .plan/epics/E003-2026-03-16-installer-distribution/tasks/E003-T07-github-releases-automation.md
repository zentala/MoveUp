---
id: E003-T07
epic: E003
status: open
original_id: T007
---
# T007: GitHub Releases CI/CD Automation

**Task ID:** T007
**Effort:** 3 hours
**Priority:** P1 (deferred, implement after T001-T006)
**Owner:** Subagent (future)
**Status:** Deferred

---

## Summary

Create `.github/workflows/release.yml` that automates installer building, signing, and publishing to GitHub Releases on git tag.

---

## Deliverables

### .github/workflows/release.yml

**Trigger:** `push` with tags matching `v*` (e.g., `v0.1.0`)

**Steps:**

1. Checkout code
2. Install Rust, Node, pnpm
3. Run tests (safety gate)
4. Build installer (`pnpm tauri:build --release`)
5. Sign binary (scripts/sign-installer.sh)
   - Uses GitHub Secrets: SIGN_CERT_PATH, SIGN_PASSWORD
6. Create GitHub Release with:
   - Release notes (from git tag)
   - Upload .exe + .msi as assets
   - Mark as latest

**Conditional:**
- Only run on Windows (runner: windows-latest)
- Only run on tags (if: startsWith(github.ref, 'refs/tags/'))

---

## Acceptance Criteria

- release.yml created
- Trigger on tags works
- Build + sign steps functional
- Assets uploaded to Release
- Release notes auto-generated

---

## Dependencies

- T001: Tauri config + signing scaffold
- T002: build-report.js (used in release job)
- Certificate: Must acquire before enabling signing step
