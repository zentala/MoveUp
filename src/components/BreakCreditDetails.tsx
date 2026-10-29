import { useId, useRef, type FC } from "react";
import type { LastBreakCreditFacts } from "@/types";
import { formatDuration } from "@/utils/format";

interface BreakCreditDetailsProps {
  facts?: LastBreakCreditFacts[] | null;
}

function displayInstant(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}

function reasonText(reason: string | null): string | null {
  switch (reason) {
    case "break_below_minimum": return "Break was shorter than the captured minimum.";
    case "balance_already_zero": return "Sitting balance was already zero.";
    case "zero_multiplier": return "The captured credit multiplier was zero.";
    case "requested_credit_rounded_to_zero": return "The captured rule requested less than one second of credit.";
    case "already_credited_prefix": return "The earlier credited amount already met this rule.";
    case null: return null;
    default: return "No credit was applied for the recorded reason.";
  }
}

/** Read-only explanation from engine-captured facts; never recalculates history. */
export const BreakCreditDetails: FC<BreakCreditDetailsProps> = ({ facts }) => {
  const headingId = useId();
  const detailsRef = useRef<HTMLDetailsElement>(null);

  if (!facts?.length) {
    return <p className="one-bar__credit-unavailable">Break credit details unavailable</p>;
  }

  return (
    <details className="one-bar__credit-details" data-testid="break-credit-details" ref={detailsRef}>
      <summary>Break credit details</summary>
      <section className="one-bar__credit-panel" role="region" aria-labelledby={headingId}>
        <header className="one-bar__credit-panel-header">
          <h2 id={headingId}>Captured break credit</h2>
          <button
            className="one-bar__credit-close"
            type="button"
            aria-label="Close break credit details"
            onClick={() => {
              detailsRef.current?.removeAttribute("open");
              detailsRef.current?.querySelector("summary")?.focus();
            }}
          >
            Close
          </button>
        </header>
        <div
          className="one-bar__credit-scroll"
          role="region"
          aria-label="Break credit entries"
          tabIndex={0}
          onKeyDown={event => {
            if (event.altKey || event.ctrlKey || event.metaKey) return;
            const body = event.currentTarget;
            const maxScroll = Math.max(0, body.scrollHeight - body.clientHeight);
            let nextScroll: number | null = null;
            switch (event.key) {
              case "ArrowDown": nextScroll = body.scrollTop + 40; break;
              case "ArrowUp": nextScroll = body.scrollTop - 40; break;
              case "PageDown": nextScroll = body.scrollTop + body.clientHeight; break;
              case "PageUp": nextScroll = body.scrollTop - body.clientHeight; break;
              case "Home": nextScroll = 0; break;
              case "End": nextScroll = maxScroll; break;
            }
            if (nextScroll !== null) {
              body.scrollTop = Math.min(maxScroll, Math.max(0, nextScroll));
              event.preventDefault();
            }
          }}
        >
          <ol className="one-bar__credit-list">
            {facts.map((fact, index) => (
              <li key={`${fact.break_started_at}-${fact.applied_at}-${index}`}>
                <dl>
                  <div><dt>Break started</dt><dd>{displayInstant(fact.break_started_at)}</dd></div>
                  <div><dt>Observed duration</dt><dd>{formatDuration(fact.observed_break_secs)}</dd></div>
                  <div><dt>Rule at application</dt><dd>{displayInstant(fact.applied_at)} · minimum {formatDuration(fact.minimum_break_secs)} · ×{fact.multiplier}</dd></div>
                  <div><dt>Earlier credit</dt><dd>{formatDuration(fact.prior_applied_credit_secs)}</dd></div>
                  <div><dt>Credit requested</dt><dd>{formatDuration(fact.requested_credit_secs)}</dd></div>
                  <div><dt>Credit applied</dt><dd>{formatDuration(fact.applied_credit_secs)}</dd></div>
                  <div><dt>Sitting balance</dt><dd>{formatDuration(fact.sitting_before_secs)} → {formatDuration(fact.sitting_after_secs)}</dd></div>
                </dl>
                {reasonText(fact.no_credit_reason) && (
                  <p className="one-bar__credit-reason">{reasonText(fact.no_credit_reason)}</p>
                )}
              </li>
            ))}
          </ol>
        </div>
      </section>
    </details>
  );
};
