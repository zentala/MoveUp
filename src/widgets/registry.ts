/**
 * registry.ts — widget registry.
 *
 * Single widget (OneBar). Registry kept for future extensibility
 * but no picker UI — OneBar is the only production widget.
 */
import type { DeskWidget } from "@/types";
import { OneBarWidget } from "./OneBarWidget";

/** The single active widget component. */
export const ActiveWidget: DeskWidget = OneBarWidget;
