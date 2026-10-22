/**
 * NotificationsSection.tsx — effective communication-profile settings.
 */
import type { FC } from "react";
import ProfileSelector from "./ProfileSelector";

const NotificationsSection: FC = () => (
  <ProfileSelector type="communication" label="Communication Profile" />
);

export default NotificationsSection;
