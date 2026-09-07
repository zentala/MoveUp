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

# eslint over src/ (config: eslint.config.mjs)
lint:
    pnpm lint

# frontend unit tests with the coverage gate (the same gate `just build` runs)
coverage:
    pnpm test:coverage

# pre-commit gate: types, lint, then both unit suites
check: typecheck lint test test-rust

# relay Worker (E022) — a standalone pnpm project in relay/, its own lockfile.
# `pnpm --dir` keeps every one of these runnable from the repo root.

# run the relay locally on :8787 (wrangler dev, local D1 + DO)
relay-dev:
    pnpm --dir relay install --frozen-lockfile && pnpm --dir relay exec wrangler dev

# relay Worker tests (workerd, via @cloudflare/vitest-pool-workers)
relay-test:
    pnpm --dir relay install --frozen-lockfile && pnpm --dir relay exec vitest run

# the relay lives in relay/, so it needs its own install and its own runner.
# full pairing + control round trip against a running relay (needs a licence)
relay-e2e license relay="http://127.0.0.1:8787":
    node scripts/relay-e2e.mjs --relay {{relay}} --license {{license}}

# deploy the relay — T16 only, and only behind consent-broker
relay-deploy:
    pnpm build && pnpm --dir relay exec wrangler deploy

# remove frontend build artifacts (src-tauri/target: use `cargo clean` deliberately)
clean:
    if (Test-Path dist) { Remove-Item -Recurse -Force dist }; exit 0
