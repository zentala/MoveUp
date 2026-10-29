/**
 * ProfileSelector.tsx — dropdown to list and switch ergonomic/communication profiles.
 *
 * Shows profile name, description, and action buttons (Edit JSON, Duplicate, Reset).
 */
import { useState, useEffect, useCallback, useRef } from "react";
import type { FC } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ProfileInfo {
  id: string;
  name: string;
  description: string;
  path: string;
}

interface ActiveProfiles {
  communication_id: string;
  ergonomic_id: string;
  effective_limits: {
    sitting_secs: number;
    standing_secs: number;
    standing_target_secs: number;
    standing_max_secs: number;
    break_min_secs: number;
    break_credit_multiplier: number;
    day_break_min_secs: number;
    posture_balance_min_sitting_secs: number;
    max_continuous_computer_secs: number;
    computer_break_reset_secs: number;
  };
  session_limit_secs: number;
  stand_limit_secs: number;
  notifications_enabled: boolean;
  dismiss_durations_mins: number[];
  current_dismiss_minutes: number;
  snooze_minutes: number | null;
  tone_shift_after_dismisses: number;
  notify_cooldowns_secs: number[];
}

function formatDuration(seconds: number): string {
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  if (!minutes) return `${remainingSeconds} sec`;
  return remainingSeconds ? `${minutes} min ${remainingSeconds} sec` : `${minutes} min`;
}

interface ProfileSelectorProps {
  /** Which profile category this selector manages. */
  type: "communication" | "ergonomic";
  /** Label shown above the dropdown. */
  label: string;
}

const SWITCH_CMD = {
  communication: "switch_communication_profile",
  ergonomic: "switch_ergonomic_profile",
} as const;

const LIST_CMD = {
  communication: "list_communication_profiles",
  ergonomic: "list_ergonomic_profiles",
} as const;

/**
 * Dropdown selector for a single profile category.
 * Fetches available profiles and active ID on mount; switches on change.
 */
const ProfileSelector: FC<ProfileSelectorProps> = ({ type, label }) => {
  const [profiles, setProfiles] = useState<ProfileInfo[]>([]);
  const [runtime, setRuntime] = useState<ActiveProfiles | null>(null);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [switching, setSwitching] = useState(false);
  const [duplicating, setDuplicating] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [duplicateName, setDuplicateName] = useState("");
  const [showDuplicate, setShowDuplicate] = useState(false);
  const readPending = useRef(false);
  const mutationPending = useRef(false);

  const activeProfile = profiles.find((p) => p.id === activeId) ?? null;

  const loadData = useCallback(async () => {
    if (readPending.current || mutationPending.current) return;
    readPending.current = true;
    setError(null);
    setLoading(true);
    try {
      const [list, active] = await Promise.all([
        invoke<ProfileInfo[]>(LIST_CMD[type]),
        invoke<ActiveProfiles | null>("get_active_profiles"),
      ]);
      setProfiles(list ?? []);
      if (active) {
        setRuntime(active);
        setActiveId(type === "communication" ? active.communication_id : active.ergonomic_id);
      } else {
        setRuntime(null);
        setError("The active runtime profile could not be confirmed. Retry reading current settings.");
      }
    } catch (err) {
      setRuntime(null);
      setError(String(err));
    } finally {
      readPending.current = false;
      setLoading(false);
    }
  }, [type]);

  useEffect(() => { loadData(); }, [loadData]);

  async function handleSwitch(id: string) {
    if (id === activeId || readPending.current || mutationPending.current) return;
    mutationPending.current = true;
    setSwitching(true);
    setError(null);
    try {
      await invoke(SWITCH_CMD[type], { id });
      setActiveId(id);
      setRuntime(null);
      mutationPending.current = false;
      await loadData();
    } catch (err) {
      setError(String(err));
    } finally {
      mutationPending.current = false;
      setSwitching(false);
    }
  }

  async function handleEdit() {
    if (!activeProfile) return;
    try {
      await invoke("open_profile_in_editor", { path: activeProfile.path });
    } catch (err) {
      setError(String(err));
    }
  }

  async function handleDuplicate() {
    if (!activeProfile || !duplicateName.trim() || readPending.current || mutationPending.current) return;
    mutationPending.current = true;
    setDuplicating(true);
    try {
      const newId = await invoke<string>("duplicate_profile", {
        sourcePath: activeProfile.path,
        newName: duplicateName.trim(),
      });
      setShowDuplicate(false);
      setDuplicateName("");
      mutationPending.current = false;
      await loadData();
      await handleSwitch(newId);
    } catch (err) {
      setError(String(err));
    } finally {
      mutationPending.current = false;
      setDuplicating(false);
    }
  }

  async function handleReset() {
    await handleSwitch("default");
  }

  return (
    <div className="settings-panel__section">
      <h3 className="settings-panel__section-title">{label}</h3>

      <div className="settings-panel__field">
        <label htmlFor={`profile-${type}`} className="settings-panel__label">
          Active profile
        </label>
        <select
          id={`profile-${type}`}
          className="settings-panel__select"
          value={activeId ?? ""}
          disabled={switching || loading || profiles.length === 0}
          onChange={(e) => handleSwitch(e.target.value)}
        >
          {activeId === null && <option value="">{loading ? "Reading active profile…" : "Active profile unavailable"}</option>}
          {profiles.map((p) => (
            <option key={p.id} value={p.id}>{p.name}</option>
          ))}
        </select>
      </div>
      <p className="settings-panel__helper">Profile changes apply immediately. Back does not undo this choice.</p>

      {activeProfile && (
        <p className="settings-panel__helper">{activeProfile.description}</p>
      )}

      {runtime?.effective_limits && type === "ergonomic" && (
        <div className="settings-panel__helper" aria-label="Effective ergonomic settings">
          <p>Applied runtime values (profile thresholds can differ from session overrides):</p>
          <ul>
            <li>Session engine sitting limit: {formatDuration(runtime.effective_limits.sitting_secs)}; current sitting alert threshold: {formatDuration(runtime.session_limit_secs)}</li>
            <li>Standing session limit: {formatDuration(runtime.stand_limit_secs)}</li>
            <li>Full-break target: {formatDuration(runtime.effective_limits.standing_target_secs)}; maximum standing: {formatDuration(runtime.effective_limits.standing_max_secs)}</li>
            <li>Break credit starts after {formatDuration(runtime.effective_limits.break_min_secs)}; configured credit multiplier: {runtime.effective_limits.break_credit_multiplier} (engine applies a value from 0 to 10)</li>
            <li>Day-break threshold: {runtime.effective_limits.day_break_min_secs === 0 ? "disabled" : formatDuration(runtime.effective_limits.day_break_min_secs)}</li>
            <li>Posture-balance threshold: {formatDuration(runtime.effective_limits.posture_balance_min_sitting_secs)} of sitting</li>
            <li>Continuous-computer reminder: {runtime.effective_limits.max_continuous_computer_secs === 0 ? "disabled" : formatDuration(runtime.effective_limits.max_continuous_computer_secs)}; away reset: {formatDuration(runtime.effective_limits.computer_break_reset_secs)}</li>
          </ul>
        </div>
      )}

      {runtime?.dismiss_durations_mins && runtime.notify_cooldowns_secs && type === "communication" && (
        <div className="settings-panel__helper" aria-label="Effective communication settings">
          <p>{runtime.notifications_enabled ? "Notifications are enabled" : "Notifications are disabled"} for the communication policy currently running.</p>
          <p>Current Dismiss delay: {runtime.current_dismiss_minutes} min. Configured Dismiss delay sequence: {runtime.dismiss_durations_mins.length ? runtime.dismiss_durations_mins.map((minutes) => `${minutes} min`).join(" → ") : "none; the current policy value is shown above"}.</p>
          <p>Snooze {runtime.snooze_minutes === null ? "is unavailable for the current policy step because its configured duration is unsupported" : `uses ${runtime.snooze_minutes} min`} without advancing the Dismiss step. Dismiss acknowledges the reminder and advances the step; the final configured delay repeats.</p>
          <p>Tone shift after {runtime.tone_shift_after_dismisses} dismissals. Automatic reminder cooldowns: {runtime.notify_cooldowns_secs.map(formatDuration).join(" → ") || "none configured"}.</p>
        </div>
      )}

      <div className="settings-panel__action-row">
        <button
          className="btn btn--secondary"
          onClick={handleEdit}
          disabled={!activeProfile}
          title="Open profile JSON in system editor"
        >
          Edit JSON
        </button>
        <button
          className="btn btn--secondary"
          onClick={() => setShowDuplicate((v) => !v)}
          disabled={!activeProfile || loading || switching || duplicating}
          title="Create a copy of this profile"
        >
          Duplicate
        </button>
        {activeId !== "default" && (
          <button
            className="btn btn--secondary"
            onClick={handleReset}
            disabled={switching || loading || duplicating}
            title="Switch back to default profile"
          >
            Reset
          </button>
        )}
      </div>

      {showDuplicate && (
        <div className="settings-panel__field settings-panel__field--inline">
          <input
            type="text"
            className="settings-panel__input"
            placeholder="New profile name"
            value={duplicateName}
            onChange={(e) => setDuplicateName(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleDuplicate()}
            autoFocus
          />
          <button
            className="btn btn--primary"
            onClick={handleDuplicate}
            disabled={!duplicateName.trim() || loading || switching || duplicating}
          >
            Create
          </button>
        </div>
      )}

      {error && <>
        <p role="alert" className="settings-panel__error">{error}</p>
        <button className="btn btn--secondary" onClick={loadData} disabled={loading || switching}>
          {loading ? "Retrying…" : "Retry reading current settings"}
        </button>
      </>}
    </div>
  );
};

export default ProfileSelector;
