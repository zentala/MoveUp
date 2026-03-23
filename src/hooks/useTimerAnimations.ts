/**
 * useTimerAnimations.ts — timer animation state for fade and shimmer effects.
 *
 * Provides:
 * - shimmer: briefly true when limitRatio crosses 1.0 (reset threshold)
 */
import { useState, useEffect, useRef } from "react";
import type { DeskState } from "@/types";

interface TimerAnimations {
  shimmer: boolean;
}

export function useTimerAnimations(
  state: DeskState,
  limitRatio: number,
): TimerAnimations {
  const [shimmer, setShimmer] = useState(false);
  const prevRatio = useRef(limitRatio);
  const prevState = useRef(state);

  useEffect(() => {
    // Shimmer when transitioning from >= 1.0 to < 1.0 (reset happened)
    const wasOver = prevRatio.current >= 1.0;
    const nowUnder = limitRatio < 1.0;
    const stateChanged = prevState.current !== state;

    if ((wasOver && nowUnder) || (stateChanged && limitRatio < 0.05)) {
      setShimmer(true);
      const timer = setTimeout(() => setShimmer(false), 800);
      prevRatio.current = limitRatio;
      prevState.current = state;
      return () => clearTimeout(timer);
    }

    prevRatio.current = limitRatio;
    prevState.current = state;
  }, [state, limitRatio]);

  return { shimmer };
}
