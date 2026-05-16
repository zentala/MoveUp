---
id: E012-T05
epic: E012
status: planned
created: 2026-05-16
branch: feat/E012-T05-catalog-live
---

# E012-T05 — Catalog tab live wiring

## Goal
Replace fixture data in the Catalog tab with a real call to `invoke('get_data_catalog')`. Add a `useDataCatalog()` hook with proper loading / error / empty states.

## Why
Catalog tab is the data analyst's reference — "what is in this system?" Wiring it live first (before Explorer) gives zentala an immediately useful artefact while T06 builds charts.

## Hook
Create `apps/desk/src/analyst/hooks/useDataCatalog.ts`:

```ts
export type DataCatalogState =
  | { status: 'loading' }
  | { status: 'ready'; data: DataCatalog }
  | { status: 'error'; message: string };

export function useDataCatalog(): DataCatalogState { … }
```

- Calls `invoke<DataCatalog>('get_data_catalog')` on mount
- Refreshes when the analyst window regains focus (`document.visibilitychange`)
- Never throws — surfaces errors via `status: 'error'`

## TypeScript types
Create `apps/desk/src/analyst/types.ts` mirroring T02 Rust types (`DataCatalog`, `DataSource`, `FieldSpec`). Add a Zod schema if the project already uses zod (grep `from 'zod'`).

## UI changes (from T03 mockup)
- `<CatalogTab>` now takes no props; uses the hook internally
- Loading state: skeleton rows (no spinner)
- Error state: inline error card with retry button (calls hook's refetch)
- Empty source list (shouldn't happen): "No data sources detected." with link to logs path
- All other visual decisions from T03 mockup carry over unchanged

## Tests
**TS unit:**
- `useDataCatalog` returns `loading` initially, transitions to `ready` after mocked `invoke` resolves
- Transitions to `error` when `invoke` rejects
- `<CatalogTab>` renders skeleton in loading, table in ready, error card in error

**Integration:**
- One Vitest test that mocks the tauri-mock layer to return the T02 fixture shape and asserts the table renders ≥ 1 row per source

## Files touched
- `apps/desk/src/analyst/hooks/useDataCatalog.ts` (new)
- `apps/desk/src/analyst/types.ts` (new)
- `apps/desk/src/analyst/CatalogTab.tsx` — replace fixture import with hook
- `apps/desk/src/analyst/CatalogTab.test.tsx` (new)

## DoD
- [ ] Opening Analyst window → Catalog tab shows real data from `get_data_catalog` within 1 s on a 7-day log dir
- [ ] Error path works (kill `commands_analyst.rs` registration locally → error card appears)
- [ ] Unit + integration tests pass; coverage ≥ 80% on the new hook/component
- [ ] Files ≤ 250 lines
- [ ] No `any` types
