/**
 * ProfileSelector.tsx — dropdown to list and switch ergonomic/communication profiles.
 *
 * Shows profile name, description, and action buttons (Edit JSON, Duplicate, Reset).
 */
import { useState, useEffect, useCallback } from "react";
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
  const [activeId, setActiveId] = useState<string>("default");
  const [switching, setSwitching] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [duplicateName, setDuplicateName] = useState("");
  const [showDuplicate, setShowDuplicate] = useState(false);

  const activeProfile = profiles.find((p) => p.id === activeId) ?? null;

  const loadData = useCallback(async () => {
    setError(null);
    try {
      const [list, active] = await Promise.all([
        invoke<ProfileInfo[]>(LIST_CMD[type]),
        invoke<ActiveProfiles | null>("get_active_profiles"),
      ]);
      setProfiles(list ?? []);
      if (active) {
        setActiveId(type === "communication" ? active.communication_id : active.ergonomic_id);
      }
    } catch (err) {
      setError(String(err));
    }
  }, [type]);

  useEffect(() => { loadData(); }, [loadData]);

  async function handleSwitch(id: string) {
    if (id === activeId) return;
    setSwitching(true);
    setError(null);
    try {
      await invoke(SWITCH_CMD[type], { id });
      setActiveId(id);
    } catch (err) {
      setError(String(err));
    } finally {
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
    if (!activeProfile || !duplicateName.trim()) return;
    try {
      const newId = await invoke<string>("duplicate_profile", {
        sourcePath: activeProfile.path,
        newName: duplicateName.trim(),
      });
      setShowDuplicate(false);
      setDuplicateName("");
      await loadData();
      await handleSwitch(newId);
    } catch (err) {
      setError(String(err));
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
          value={activeId}
          disabled={switching || profiles.length === 0}
          onChange={(e) => handleSwitch(e.target.value)}
        >
          {profiles.length === 0 && (
            <option value="default">default</option>
          )}
          {profiles.map((p) => (
            <option key={p.id} value={p.id}>{p.name}</option>
          ))}
        </select>
      </div>

      {activeProfile && (
        <p className="settings-panel__helper">{activeProfile.description}</p>
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
          disabled={!activeProfile}
          title="Create a copy of this profile"
        >
          Duplicate
        </button>
        {activeId !== "default" && (
          <button
            className="btn btn--secondary"
            onClick={handleReset}
            disabled={switching}
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
            disabled={!duplicateName.trim()}
          >
            Create
          </button>
        </div>
      )}

      {error && <p className="settings-panel__error">{error}</p>}
    </div>
  );
};

export default ProfileSelector;
