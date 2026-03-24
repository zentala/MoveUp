---
id: E003-T06
epic: E003
status: done
original_id: T006
---
# T006: Performance Optimization Guidelines

**Task ID:** T006
**Effort:** 2 hours
**Priority:** P2 (informational, depends on T003)
**Owner:** Subagent

---

## Summary

Create docs/OPTIMIZATION_GUIDE.md with performance baselines, optimization strategies, and regression detection checklist.

---

## Deliverables

### docs/OPTIMIZATION_GUIDE.md

**Sections:**

1. **Current Baselines** (from T003 memory profiling)
   - Installer: 60-70 MB
   - Peak memory: ~250 MB
   - Stable memory: ~180 MB
   - Breakdown by component

2. **Bundle Size Strategies**
   - Tree-shake unused plugins
   - React code-splitting
   - Strip debug symbols
   - LTO in Cargo.toml

3. **Runtime Memory Strategies**
   - Lazy-load React components
   - Batch database updates
   - Archive old sessions
   - Memory pooling

4. **Database Optimization**
   - VACUUM on startup
   - Indexes on hot columns
   - Partition by date

5. **Regression Detection Checklist**
   - New npm package? Benchmark it
   - New Rust dependency? Check size
   - Before merge: check growth
   - Before release: run `pnpm test:perf`

---

## Acceptance Criteria

- Baselines clearly stated
- Strategies are actionable
- Checklist is testable
- Links to tools/guides work

---

## Depends On

- T003: Memory profiling (provides baseline data)
