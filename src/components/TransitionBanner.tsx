/**
 * TransitionBanner.tsx — shows transition info for 30s after a state change.
 *
 * Displays break duration + credit label after Standing->Sitting,
 * or sitting duration after Sitting->Standing/Walking/Away.
 */
import type { FC } from "react";
import type { TransitionInfo } from "@/hooks/useDesk";

/** Human-readable label for the break credit applied. */
function breakCreditLabel(credit: string, secs: number): string {
  const min = Math.round(secs / 60);
  switch (credit) {
    case "full":
      return `Stood ${min} min — full reset`;
    case "partial":
      return `Stood ${min} min — session reduced`;
    case "none":
      return `Stood ${min} min — session continues`;
    default:
      return "";
  }
}

/** Human-readable label for sitting duration on leaving Sitting state. */
function sittingLabel(secs: number): string {
  const min = Math.round(secs / 60);
  return `Sat for ${min} min`;
}

interface TransitionBannerProps {
  transition: TransitionInfo;
}

/** Renders a transition message banner. Auto-cleared by parent after 30s. */
const TransitionBanner: FC<TransitionBannerProps> = ({ transition }) => {
  const { transitionTo, lastBreakSecs, lastSittingSecs, breakCredit } = transition;

  let message = "";

  if (transitionTo === "Sitting" && lastBreakSecs > 0) {
    message = breakCreditLabel(breakCredit, lastBreakSecs);
  } else if (transitionTo !== "Sitting" && lastSittingSecs > 0) {
    message = sittingLabel(lastSittingSecs);
  }

  if (!message) return null;

  return (
    <div className="transition-banner" data-testid="transition-banner">
      {message}
    </div>
  );
};

export default TransitionBanner;
