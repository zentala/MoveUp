(() => {
  try {
    const emptyQuery = () => ({ outcome: "not_observed", last_result_ms: null, last_success_ms: null });
    const memory = window.__MOVEUP_UI_DIAGNOSTICS__;
    if (memory !== undefined && (!memory || memory.schema_version !== 1)) return { collection_error: "invalid_snapshot" };
    const snapshot = memory === undefined ? {
      schema_version: 1, boot: "unknown", view: "other",
      queries: { dashboard: emptyQuery(), today_summary: emptyQuery() }, last_error: null,
    } : memory;
    // Validate nullable numbers BEFORE WebView JSON serialization turns NaN/Infinity into null.
    const nullableInteger = (value, min, max) => value === null ||
      (Number.isSafeInteger(value) && value >= min && value <= max);
    for (const value of [snapshot.queries.dashboard, snapshot.queries.today_summary]) {
      if (!nullableInteger(value.last_result_ms, 0, 8640000000000000) ||
          !nullableInteger(value.last_success_ms, 0, 8640000000000000)) return { collection_error: "invalid_snapshot" };
    }
    if (snapshot.last_error !== null && snapshot.last_error.location !== null) {
      const { line, column } = snapshot.last_error.location;
      if (!nullableInteger(line, 1, 1000000000) || !nullableInteger(column, 1, 1000000000) ||
          (line === null) !== (column === null)) return { collection_error: "invalid_snapshot" };
    }
    const hint = value => ({ outcome: value.outcome, last_result_ms: value.last_result_ms, last_success_ms: value.last_success_ms });
    const error = snapshot.last_error === null ? null : {
      source: snapshot.last_error.source, kind: snapshot.last_error.kind,
      location: snapshot.last_error.location === null ? null : {
        module: snapshot.last_error.location.module,
        line: snapshot.last_error.location.line, column: snapshot.last_error.location.column,
      },
    };
    const root = document.getElementById("root");
    const rectangle = root ? root.getBoundingClientRect() : { width: 0, height: 0 };
    const style = root ? window.getComputedStyle(root) : null;
    return {
      schema_version: 1,
      ready: ["loading", "interactive", "complete"].includes(document.readyState) ? document.readyState : "unknown",
      visible: document.visibilityState === "visible", diagnostics_available: memory !== undefined,
      boot: snapshot.boot, view: snapshot.view, root_present: root !== null,
      root_children: root ? root.childElementCount : 0,
      width_px: Math.ceil(rectangle.width), height_px: Math.ceil(rectangle.height),
      display_none: style !== null && style.display === "none",
      visibility_hidden: style !== null && style.visibility === "hidden",
      zero_opacity: style !== null && style.opacity === "0",
      queries: { dashboard: hint(snapshot.queries.dashboard), today_summary: hint(snapshot.queries.today_summary) },
      last_error: error,
    };
  } catch { return { collection_error: "collection_failed" }; }
})()
