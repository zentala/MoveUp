import { Component, type ErrorInfo, type ReactNode } from "react";
import { reportRenderFailure } from "../uiDiagnostics";

type Props = {
  children: ReactNode;
};

type State = {
  failed: boolean;
};

export default class AppErrorBoundary extends Component<Props, State> {
  state: State = { failed: false };

  static getDerivedStateFromError(): State {
    return { failed: true };
  }

  componentDidCatch(error: Error, _errorInfo: ErrorInfo): void {
    reportRenderFailure(error);
  }

  render(): ReactNode {
    if (this.state.failed) {
      return (
        <div
          role="alert"
          style={{
            boxSizing: "border-box",
            width: "100vw",
            height: "100vh",
            display: "grid",
            placeItems: "center",
            padding: 12,
            background: "var(--panel-base, #161616)",
            color: "var(--ink-primary, #f4f4f4)",
            fontFamily:
              'var(--font-mono, ui-monospace, "Cascadia Code", "Fira Mono", Consolas, monospace)',
            fontSize: 13,
            lineHeight: 1.5,
          }}
        >
          View unavailable.
        </div>
      );
    }

    return this.props.children;
  }
}
