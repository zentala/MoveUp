/**
 * OneBarCoach.tsx — single context-dependent coaching sentence.
 *
 * The coach tells the user what to DO, not statistics.
 * One sentence max, changes based on state and limit ratio.
 */
import type { FC } from "react";
import type { WidgetProps } from "@/types";

/** Selects the appropriate coach message based on current state and progress. */
export function selectCoachMessage(props: WidgetProps): string {
  const { state, limitRatio, limitRemaining, limitSecs, breakResetProgress } =
    props;

  if (state === "Standing" || state === "Walking") {
    if (breakResetProgress >= 1.0) {
      const limitMin = Math.floor(limitSecs / 60);
      return `Reset! You can sit down — you have a full ${limitMin} min.`;
    }
    const toResetSecs = Math.max(
      0,
      props.breakResetThreshold * (1 - breakResetProgress),
    );
    const toResetMin = Math.ceil(toResetSecs / 60);
    return `${toResetMin} min left until reset.`;
  }

  if (state === "Away") {
    if (breakResetProgress >= 1.0) {
      const limitMin = Math.floor(limitSecs / 60);
      return `Reset! Come back and you have a full ${limitMin} min.`;
    }
    const toResetSecs = Math.max(
      0,
      props.breakResetThreshold * (1 - breakResetProgress),
    );
    const toResetMin = Math.ceil(toResetSecs / 60);
    return `Break counts. ${toResetMin} min to reset.`;
  }

  // Sitting states
  if (limitRatio >= 1.0) {
    const overtimeSecs = Math.abs(limitRemaining);
    const overtimeMin = Math.ceil(overtimeSecs / 60);
    return `Limit exceeded by ${overtimeMin} min. Losing points.`;
  }
  if (limitRatio >= 0.8) {
    const remainingMin = Math.ceil(Math.max(0, limitRemaining) / 60);
    return `Stand up within ${remainingMin} min.`;
  }
  if (limitRatio >= 0.5) {
    return `Half limit used. ${props.todayChanges} changes today.`;
  }
  return "On track.";
}

/** Renders the one-line coach sentence at the bottom of the widget. */
export const OneBarCoach: FC<WidgetProps> = (props) => {
  const message = selectCoachMessage(props);

  return (
    <div className="one-bar__coach" data-testid="one-bar-coach">
      <span className="one-bar__coach-indicator" />
      <span className="one-bar__coach-text">{message}</span>
    </div>
  );
};
