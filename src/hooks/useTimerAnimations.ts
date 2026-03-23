/**
 * useTimerAnimations.ts — hook for timer visual transitions.
 *
 * Provides fade (on state change) and shimmer (on break credit reset) flags.
 */
import { useState, useEffect, useRef } from "react";
import type { DeskState } from "@/types";

interface TimerAnimations {
  fade: boolean;
  shimmer: boolean;
}

export function useTimerAnimations(state: DeskState, progress: number): TimerAnimations {
  const [fade, setFade] = useState(false);
  const [shimmer, setShimmer] = useState(false);
  const prevStateRef = useRef<DeskState>(state);
  const prevProgressRef = useRef<number>(progress);

  useEffect(() => {
    if (prevStateRef.current !== state) {
      prevStateRef.current = state;
      setFade(true);
      const timer = setTimeout(() => setFade(false), 300);
      return () => clearTimeout(timer);
    }
  }, [state]);

  useEffect(() => {
    const prev = prevProgressRef.current;
    prevProgressRef.current = progress;
    if (prev >= 1.0 && progress < 1.0) {
      setShimmer(true);
      const timer = setTimeout(() => setShimmer(false), 600);
      return () => clearTimeout(timer);
    }
  }, [progress]);

  return { fade, shimmer };
}
