# One command vocabulary for this repo (see ~/.claude/rules/just.md).
# Every recipe is a one-line wrapper around a script that already exists in
# package.json — no build logic lives here.
set windows-shell := ["pwsh", "-NoProfile", "-Command"]

# list targets
default:
    @just --list

# install dependencies
setup:
    pnpm install --frozen-lockfile

# run the web frontend in dev mode (Tauri shell: `pnpm tauri:dev`)
dev:
    pnpm dev

# typecheck, test, and build the frontend bundle
build:
    pnpm build

# frontend unit tests
test:
    pnpm test:unit

# Rust unit tests (cargo test in src-tauri/)
test-rust:
    pnpm test:unit:rust

# types only
typecheck:
    pnpm typecheck

# eslint over src/ — broken today (no eslint dependency/config), fixed in E018
lint:
    pnpm lint

# pre-commit gate: same set as package.json's test:all, plus typecheck
check: typecheck test test-rust

# remove frontend build artifacts (src-tauri/target: use `cargo clean` deliberately)
clean:
    if (Test-Path dist) { Remove-Item -Recurse -Force dist }; exit 0
