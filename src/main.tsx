/**
 * main.tsx — entry point for the Desk application.
 */
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import AppErrorBoundary from "./components/AppErrorBoundary";
import { startUiDiagnostics } from "./uiDiagnostics";
import App from "./App";

const rootElement = document.getElementById("root");
if (!rootElement) throw new Error("Root element #root not found");

startUiDiagnostics();

createRoot(rootElement).render(
  <StrictMode>
    <AppErrorBoundary>
      <App />
    </AppErrorBoundary>
  </StrictMode>,
);
