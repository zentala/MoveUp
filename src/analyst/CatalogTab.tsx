/**
 * CatalogTab.tsx — Sortable + filterable table of data sources.
 *
 * Data source: either an explicit `data` prop (mockup/test) or the
 * `useDataCatalog()` hook (live Tauri invoke). When no prop is passed,
 * the component fetches the catalog on mount.
 */
import { useMemo, useState } from "react";
import { chartColors } from "./charts/chart-utils";
import { useDataCatalog } from "./hooks/useDataCatalog";
import type { DataCatalog, DataSource } from "./types/catalog";

export interface CatalogTabProps {
  /** Optional explicit catalog (mockup / test). When omitted, the hook fetches live. */
  data?: DataCatalog;
}

type SortKey = "name" | "kind" | "fields";

const thStyle: React.CSSProperties = {
  textAlign: "left",
  padding: "6px 8px",
  fontSize: 11,
  textTransform: "uppercase",
  letterSpacing: 0.4,
  color: chartColors.subtext,
  borderBottom: `1px solid ${chartColors.gridline}`,
  cursor: "pointer",
  userSelect: "none",
};

const tdStyle: React.CSSProperties = {
  padding: "8px",
  fontSize: 12,
  borderBottom: "1px solid #1a1a26",
  verticalAlign: "top",
};

export function CatalogTab({ data }: CatalogTabProps) {
  // Skip the fetch when an explicit `data` prop is provided (mockup mode).
  const fetched = useDataCatalog(data !== undefined);
  const state = data
    ? ({ status: "ready", data, refetch: fetched.refetch } as const)
    : fetched;

  if (state.status === "loading") {
    return (
      <div data-testid="catalog-tab-loading" style={{ color: chartColors.subtext, fontSize: 12 }}>
        Loading catalog…
      </div>
    );
  }
  if (state.status === "error") {
    return (
      <div
        data-testid="catalog-tab-error"
        style={{
          color: "#f44336",
          fontSize: 12,
          border: "1px solid #2a2a3a",
          padding: 12,
          borderRadius: 4,
          display: "flex",
          alignItems: "center",
          gap: 12,
        }}
      >
        <span style={{ flex: 1 }}>Failed to load catalog: {state.error}</span>
        <button
          type="button"
          onClick={state.refetch}
          style={{
            background: "transparent",
            border: "1px solid #f44336",
            color: "#f44336",
            padding: "4px 12px",
            borderRadius: 4,
            fontSize: 11,
            cursor: "pointer",
            fontFamily: "inherit",
          }}
        >
          Retry
        </button>
      </div>
    );
  }
  return <CatalogTable sources={state.data.sources} />;
}

interface CatalogTableProps {
  sources: DataSource[];
}

function CatalogTable({ sources }: CatalogTableProps) {
  const [filter, setFilter] = useState("");
  const [sortKey, setSortKey] = useState<SortKey>("name");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");

  const filtered = useMemo(() => {
    const q = filter.trim().toLowerCase();
    const arr = q
      ? sources.filter(
          (s) =>
            s.name.toLowerCase().includes(q) ||
            s.kind.toLowerCase().includes(q) ||
            s.location.toLowerCase().includes(q),
        )
      : [...sources];
    arr.sort((a, b) => {
      let cmp = 0;
      if (sortKey === "fields") cmp = a.fields.length - b.fields.length;
      else cmp = String(a[sortKey]).localeCompare(String(b[sortKey]));
      return sortDir === "asc" ? cmp : -cmp;
    });
    return arr;
  }, [sources, filter, sortKey, sortDir]);

  function toggleSort(key: SortKey) {
    if (sortKey === key) setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    else {
      setSortKey(key);
      setSortDir("asc");
    }
  }

  return (
    <div data-testid="catalog-tab">
      <div style={{ marginBottom: 12 }}>
        <input
          type="search"
          placeholder="Filter sources by name, kind, or location..."
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          style={{
            background: "#11111c",
            border: "1px solid #2a2a3a",
            color: chartColors.text,
            padding: "6px 10px",
            borderRadius: 4,
            fontSize: 12,
            width: 360,
            fontFamily: "inherit",
          }}
        />
      </div>
      <table
        style={{ width: "100%", borderCollapse: "collapse", color: chartColors.text }}
      >
        <thead>
          <tr>
            <th style={thStyle} onClick={() => toggleSort("name")}>
              Name {sortKey === "name" ? (sortDir === "asc" ? "▲" : "▼") : ""}
            </th>
            <th style={thStyle} onClick={() => toggleSort("kind")}>
              Kind {sortKey === "kind" ? (sortDir === "asc" ? "▲" : "▼") : ""}
            </th>
            <th style={thStyle}>Location</th>
            <th style={thStyle}>Retention</th>
            <th style={thStyle} onClick={() => toggleSort("fields")}>
              Fields {sortKey === "fields" ? (sortDir === "asc" ? "▲" : "▼") : ""}
            </th>
          </tr>
        </thead>
        <tbody>
          {filtered.map((s) => (
            <tr key={s.id}>
              <td style={{ ...tdStyle, fontWeight: 600 }}>
                {s.name}
                {s.description ? (
                  <div
                    style={{
                      fontWeight: 400,
                      fontSize: 11,
                      color: chartColors.subtext,
                      marginTop: 2,
                    }}
                  >
                    {s.description}
                  </div>
                ) : null}
              </td>
              <td style={tdStyle}>{s.kind}</td>
              <td
                style={{ ...tdStyle, fontFamily: "monospace", color: chartColors.subtext }}
              >
                {s.location}
              </td>
              <td style={tdStyle}>{s.retention}</td>
              <td style={tdStyle}>
                <details>
                  <summary style={{ cursor: "pointer", color: chartColors.subtext }}>
                    {s.fields.length} fields
                  </summary>
                  <ul style={{ margin: "6px 0 0 16px", padding: 0, fontSize: 11 }}>
                    {s.fields.map((f) => (
                      <li key={f.name} style={{ marginBottom: 3 }}>
                        <code style={{ color: chartColors.primary }}>{f.name}</code>{" "}
                        <span style={{ color: chartColors.subtext }}>{f.type}</span>
                        {f.description ? (
                          <div
                            style={{
                              fontSize: 11,
                              color: chartColors.subtext,
                              marginLeft: 0,
                              opacity: 0.85,
                            }}
                          >
                            {f.description}
                          </div>
                        ) : null}
                      </li>
                    ))}
                  </ul>
                </details>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
