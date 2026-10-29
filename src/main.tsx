/**
 * main.tsx — entry point for the Desk application.
 */
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import AppErrorBoundary from "./components/AppErrorBoundary";
import { reportImportFailure, startUiDiagnostics } from "./uiDiagnostics";

// A static App import evaluates its dependencies before this bootstrap can run.
startUiDiagnostics();

const rootElement = document.getElementById("root");
if (!rootElement) throw new Error("Root element #root not found");

void import("./App")
  .then(({ default: App }) => {
    createRoot(rootElement).render(
      <StrictMode>
        <AppErrorBoundary>
          <App />
        </AppErrorBoundary>
      </StrictMode>,
    );
  })
  .catch(reportImportFailure);
