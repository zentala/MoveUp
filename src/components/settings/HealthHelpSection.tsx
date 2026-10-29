import type { FC } from "react";
import type { MouseEvent } from "react";

const PRIVACY_GUIDE = "https://github.com/zentala/MoveUp/blob/main/docs/PRIVACY.md";
const REMOTE_DISPLAY_GUIDE = "https://github.com/zentala/MoveUp/blob/main/docs/REMOTE_DISPLAY.md";

function openGuide(event: MouseEvent<HTMLAnchorElement>): void {
  event.preventDefault();
  window.open(event.currentTarget.href, "_blank");
}

/** Optional, privacy-aware orientation for the existing Health integrations. */
export const HealthHelpSection: FC = () => (
  <section className="settings-panel__section" aria-labelledby="health-help-title">
    <h3 id="health-help-title" className="settings-panel__section-title">Optional health readings</h3>
    <p className="settings-panel__helper">
      Health readings are optional and do not affect desk-state estimates or break credit. Without a configured source, MoveUp continues its regular desk tracking.
    </p>
    <p className="settings-panel__helper">
      <strong>Google Fit:</strong> off by default. When configured, MoveUp requests step data from Google; your desk history is not sent.
    </p>
    <p className="settings-panel__helper">
      <strong>Phone readings:</strong> an existing phone source can provide today’s values through the Remote Display feature.
    </p>
    <p className="settings-panel__helper">
      Read the existing <a href={PRIVACY_GUIDE} target="_blank" rel="noreferrer" onClick={openGuide}>Privacy guide</a> for data details and the <a href={REMOTE_DISPLAY_GUIDE} target="_blank" rel="noreferrer" onClick={openGuide}>Remote Display guide</a> for optional phone setup. No account or setup is required to use MoveUp.
    </p>
  </section>
);

export default HealthHelpSection;
