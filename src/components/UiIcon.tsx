import type { LucideIcon } from "lucide-react";

export interface UiIconProps {
  icon: LucideIcon;
  size?: number;
}

export function UiIcon({ icon: Icon, size = 16 }: UiIconProps) {
  return <Icon aria-hidden="true" focusable="false" size={size} strokeWidth={1.5} />;
}
