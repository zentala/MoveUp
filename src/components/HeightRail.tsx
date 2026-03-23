/**
 * HeightRail.tsx — vertical height gauge showing desk position in its physical range.
 *
 * Signature element: makes the desk's physical height tangible in the UI.
 * An amber indicator dot slides along the 3px rail as the desk moves.
 * Sitting zone (low) sits near the bottom; standing zone (high) near the top.
 *
 * Triggers a subtle pulse animation when the user stands up (Sitting→Standing).
 */
import { useEffect, useRef, useState, type FC, type ReactNode } from "react";
import type { DeskState } from "@/types";

/** Minimum expected desk height (sitting low) in cm. */
const MIN_CM = 60;
/** Maximum expected desk height (standing high) in cm. */
const MAX_CM = 130;

interface HeightRailProps {
  /** Current desk height in cm. 0 = no reading yet (hides indicator). */
  deskHeightCm: number;
  /** Current ergonomic state (used to detect Sitting→Standing transition). */
  state: DeskState | null;
  /** Content rendered to the right of the rail. */
  children: ReactNode;
}

/**
 * Wraps children with a thin vertical amber rail on the left edge.
 * The amber dot indicates current desk height within the sit/stand range.
 * Pulses when user transitions from Sitting to Standing.
 */
const HeightRail: FC<HeightRailProps> = ({ deskHeightCm, state, children }) => {
  const prevStateRef = useRef<DeskState | null>(null);
  const [isPulsing, setIsPulsing] = useState(false);

  // Detect Sitting→Standing transition and trigger pulse
  useEffect(() => {
    const prevState = prevStateRef.current;
    if (prevState === "Sitting" && state === "Standing") {
      setIsPulsing(true);
      const timer = setTimeout(() => setIsPulsing(false), 600);
      return () => clearTimeout(timer);
    }
    prevStateRef.current = state;
  }, [state]);

  const clamped = Math.max(MIN_CM, Math.min(MAX_CM, deskHeightCm));
  // 0% = bottom (sitting), 100% = top (standing)
  const pct = deskHeightCm > 0
    ? ((clamped - MIN_CM) / (MAX_CM - MIN_CM)) * 100
    : -1; // -1 = no reading — hide indicator

  return (
    <div className="height-rail-wrap">
      <div className="height-rail" title={deskHeightCm > 0 ? `${deskHeightCm.toFixed(0)} cm` : undefined}>
        {pct >= 0 && (
          <div
            className={`height-rail__indicator ${isPulsing ? "height-rail__indicator--pulse" : ""}`}
            style={{ bottom: `${pct}%` }}
          />
        )}
      </div>
      <div className="height-rail__content">
        {children}
      </div>
    </div>
  );
};

export default HeightRail;
