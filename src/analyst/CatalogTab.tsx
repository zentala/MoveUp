/**
 * CatalogTab.tsx — Sortable + filterable table of data sources.
 */
import { useMemo, useState } from "react";
import type { CatalogSource } from "@/test/analyst-fixtures";
import { chartColors } from "./charts/chart-utils";

export interface CatalogTabProps {
  sources: CatalogSource[];
}

type SortKey = "name" | "kind" | "rowCount" | "bytes";

function formatBytes(b: number): string {
  if (b === 0) return "—";
  if (b < 1024) return `${b} B`;
  if (b < 1024 * 1024) return `${(b / 1024).toFixed(1)} KB`;
  return `${(b / 1024 / 1024).toFixed(1)} MB`;
}

function formatRows(n: number): string {
  if (n === 0) return "stream";
  return n.toLocaleString();
}

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

export function CatalogTab({ sources }: CatalogTabProps) {
  const [filter, setFilter] = useState("");
  const [sortKey, setSortKey] = useState<SortKey>("name");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");

  const filtered = useMemo(() => {
    const q = filter.trim().toLowerCase();
    const arr = q
      ? sources.filter(
          (s) =>
            s.name.toLowerCase().includes(q) ||
            s.kind.includes(q) ||
            s.location.toLowerCase().includes(q),
        )
      : [...sources];
    arr.sort((a, b) => {
      const av = a[sortKey];
      const bv = b[sortKey];
      let cmp = 0;
      if (typeof av === "number" && typeof bv === "number") cmp = av - bv;
      else cmp = String(av).localeCompare(String(bv));
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
      <table style={{ width: "100%", borderCollapse: "collapse", color: chartColors.text }}>
        <thead>
          <tr>
            <th style={thStyle} onClick={() => toggleSort("name")}>
              Name {sortKey === "name" ? (sortDir === "asc" ? "▲" : "▼") : ""}
            </th>
            <th style={thStyle} onClick={() => toggleSort("kind")}>
              Kind {sortKey === "kind" ? (sortDir === "asc" ? "▲" : "▼") : ""}
            </th>
            <th style={thStyle}>Location</th>
            <th style={thStyle} onClick={() => toggleSort("rowCount")}>
              Rows {sortKey === "rowCount" ? (sortDir === "asc" ? "▲" : "▼") : ""}
            </th>
            <th style={thStyle} onClick={() => toggleSort("bytes")}>
              Size {sortKey === "bytes" ? (sortDir === "asc" ? "▲" : "▼") : ""}
            </th>
            <th style={thStyle}>Retention</th>
            <th style={thStyle}>Fields</th>
          </tr>
        </thead>
        <tbody>
          {filtered.map((s) => (
            <tr key={s.id}>
              <td style={{ ...tdStyle, fontWeight: 600 }}>{s.name}</td>
              <td style={tdStyle}>{s.kind}</td>
              <td style={{ ...tdStyle, fontFamily: "monospace", color: chartColors.subtext }}>
                {s.location}
              </td>
              <td style={tdStyle}>{formatRows(s.rowCount)}</td>
              <td style={tdStyle}>{formatBytes(s.bytes)}</td>
              <td style={tdStyle}>{s.retention}</td>
              <td style={tdStyle}>
                <details>
                  <summary style={{ cursor: "pointer", color: chartColors.subtext }}>
                    {s.fields.length} fields
                  </summary>
                  <ul style={{ margin: "6px 0 0 16px", padding: 0, fontSize: 11 }}>
                    {s.fields.map((f) => (
                      <li key={f.name}>
                        <code style={{ color: chartColors.primary }}>{f.name}</code>{" "}
                        <span style={{ color: chartColors.subtext }}>{f.type}</span>
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
