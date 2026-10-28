import { useCallback, useEffect, useRef, useState } from "react";
import type { DailyKpi, SessionRow, SnapshotRow } from "@/test/analyst-fixtures";
import { CatalogTab } from "./CatalogTab";
import { ExplorerTab } from "./ExplorerTab";
import type { DateRange } from "./DateRangePicker";
import { chartColors } from "./charts/chart-utils";
import type { DataCatalog } from "./types/catalog";
import { OPEN_ANALYST_DAY_KEY, parseOpenDayRequest, rangeForOpenDay, type OpenDayRequest } from "./open-day";
import type { LiveObservation } from "@/generated/LiveObservation";
import type { QueryStatus } from "@/hooks/useDeskTypes";

export interface AnalystWindowProps {
  catalog?: DataCatalog;
  snapshots?: SnapshotRow[];
  sessions?: SessionRow[];
  kpis?: DailyKpi[];
  liveObservation?: LiveObservation | null;
  dashboardRefresh?: QueryStatus;
  defaultRange: DateRange;
}

type TabId = "catalog" | "explorer";
const RANGE_STORAGE_KEY = "desk:analyst:range";
const TAB_ORDER: TabId[] = ["catalog", "explorer"];

function loadStoredRange(): DateRange | null {
  if (typeof localStorage === "undefined") return null;
  try {
    const raw = localStorage.getItem(RANGE_STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<DateRange>;
    return typeof parsed.from === "string" && typeof parsed.to === "string" ? { from: parsed.from, to: parsed.to } : null;
  } catch {
    return null;
  }
}

function saveStoredRange(range: DateRange): void {
  if (typeof localStorage === "undefined") return;
  try { localStorage.setItem(RANGE_STORAGE_KEY, JSON.stringify(range)); } catch { /* Best-effort preference only. */ }
}

const tabButton = (active: boolean): React.CSSProperties => ({
  background: "transparent", border: "none", color: active ? chartColors.primary : chartColors.subtext,
  borderBottom: `2px solid ${active ? chartColors.primary : "transparent"}`, padding: "8px 16px",
  fontSize: 13, cursor: "pointer", fontFamily: "inherit",
});

export function AnalystWindow({ catalog, snapshots, sessions, kpis, liveObservation = null,
  dashboardRefresh, defaultRange }: AnalystWindowProps) {
  const [tab, setTab] = useState<TabId>("explorer");
  const [focusedTab, setFocusedTab] = useState<TabId>("explorer");
  const [range, setRange] = useState<DateRange>(() => loadStoredRange() ?? defaultRange);
  const [pendingOpenDay, setPendingOpenDay] = useState<OpenDayRequest | null>(null);
  const pendingRequestId = useRef<string | null>(null);
  const tabRefs = useRef<Record<TabId, HTMLButtonElement | null>>({ catalog: null, explorer: null });

  const focusTab = (next: TabId) => {
    setFocusedTab(next);
    tabRefs.current[next]?.focus();
  };
  const onTabKeyDown = (event: React.KeyboardEvent<HTMLButtonElement>, current: TabId) => {
    const index = TAB_ORDER.indexOf(current);
    const next = event.key === "ArrowRight" ? TAB_ORDER[(index + 1) % TAB_ORDER.length]
      : event.key === "ArrowLeft" ? TAB_ORDER[(index + TAB_ORDER.length - 1) % TAB_ORDER.length]
        : event.key === "Home" ? TAB_ORDER[0] : event.key === "End" ? TAB_ORDER[TAB_ORDER.length - 1] : null;
    if (next) { event.preventDefault(); focusTab(next); }
    else if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      setTab(current);
      setFocusedTab(current);
    }
  };

  useEffect(() => {
    const removeInvalidIfStillCurrent = (raw: string) => {
      try { if (localStorage.getItem(OPEN_ANALYST_DAY_KEY) === raw) localStorage.removeItem(OPEN_ANALYST_DAY_KEY); }
      catch { /* Preserve malformed storage if access is denied. */ }
    };
    const receiveLatest = () => {
      let raw: string | null;
      try { raw = localStorage.getItem(OPEN_ANALYST_DAY_KEY); } catch { return; }
      if (!raw) return;
      const request = parseOpenDayRequest(raw);
      if (!request) { removeInvalidIfStillCurrent(raw); return; }
      if (pendingRequestId.current === request.requestId) return;
      pendingRequestId.current = request.requestId;
      setTab("explorer");
      setFocusedTab("explorer");
      setRange((current) => rangeForOpenDay(current, request.day));
      setPendingOpenDay(request);
    };
    receiveLatest();
    const onStorage = (event: StorageEvent) => {
      if (event.key === OPEN_ANALYST_DAY_KEY || event.key === null) receiveLatest();
    };
    window.addEventListener("storage", onStorage);
    return () => window.removeEventListener("storage", onStorage);
  }, []);

  const acknowledgeOpenDay = useCallback((requestId: string) => {
    try {
      const raw = localStorage.getItem(OPEN_ANALYST_DAY_KEY);
      const current = parseOpenDayRequest(raw);
      if (!raw || current?.requestId !== requestId) return;
      localStorage.removeItem(OPEN_ANALYST_DAY_KEY);
      if (localStorage.getItem(OPEN_ANALYST_DAY_KEY) !== null) return;
      if (pendingRequestId.current === requestId) pendingRequestId.current = null;
      setPendingOpenDay((pending) => pending?.requestId === requestId ? null : pending);
    } catch {
      // Failed storage access is not acknowledgement; leave the request pending.
    }
  }, []);

  useEffect(() => { saveStoredRange(range); }, [range]);

  return <div style={{ minHeight: "100vh", background: chartColors.background, color: chartColors.text,
    fontFamily: "var(--font-mono, ui-monospace, 'Cascadia Code', 'Fira Mono', Consolas, monospace)",
    padding: 16, boxSizing: "border-box" }} data-testid="analyst-window">
    <header style={{ marginBottom: 16 }}>
      <h1 style={{ margin: 0, fontSize: 20, fontWeight: 600 }}>Analyst</h1>
      <p style={{ margin: "2px 0 0", fontSize: 12, color: chartColors.subtext }}>Inspect every data source the desk app produces, then explore it.</p>
    </header>
    <nav role="tablist" aria-label="Analyst views" style={{ borderBottom: `1px solid ${chartColors.gridline}`, marginBottom: 16 }}>
      <button ref={(node) => { tabRefs.current.catalog = node; }} id="analyst-tab-catalog" role="tab" aria-controls="analyst-panel-catalog" aria-selected={tab === "catalog"} tabIndex={focusedTab === "catalog" ? 0 : -1} style={tabButton(tab === "catalog")} onFocus={() => setFocusedTab("catalog")} onClick={() => { setTab("catalog"); setFocusedTab("catalog"); }} onKeyDown={(event) => onTabKeyDown(event, "catalog")}>Catalog</button>
      <button ref={(node) => { tabRefs.current.explorer = node; }} id="analyst-tab-explorer" role="tab" aria-controls="analyst-panel-explorer" aria-selected={tab === "explorer"} tabIndex={focusedTab === "explorer" ? 0 : -1} style={tabButton(tab === "explorer")} onFocus={() => setFocusedTab("explorer")} onClick={() => { setTab("explorer"); setFocusedTab("explorer"); }} onKeyDown={(event) => onTabKeyDown(event, "explorer")}>Explorer</button>
    </nav>
    {tab === "catalog" ? <section id="analyst-panel-catalog" role="tabpanel" aria-labelledby="analyst-tab-catalog" tabIndex={0}><CatalogTab data={catalog} /></section> :
      <section id="analyst-panel-explorer" role="tabpanel" aria-labelledby="analyst-tab-explorer" tabIndex={0}><ExplorerTab snapshots={snapshots} sessions={sessions} kpis={kpis}
        liveObservation={liveObservation} dashboardRefresh={dashboardRefresh}
        range={range} onRangeChange={setRange} openDayRequest={pendingOpenDay} onOpenDayHandled={acknowledgeOpenDay} /></section>}
  </div>;
}
