/**
 * welcome.tsx — entry point for the welcome popup window.
 *
 * Rendered in a separate Tauri WebviewWindow (label: "welcome"),
 * not inside the main App window.
 */
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { WelcomePopup } from "./components/WelcomePopup";

const rootElement = document.getElementById("root");
if (!rootElement) throw new Error("Root element #root not found");

createRoot(rootElement).render(
  <StrictMode>
    <WelcomePopup />
  </StrictMode>,
);
