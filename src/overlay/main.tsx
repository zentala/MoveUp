/**
 * overlay/main.tsx — Canvas-based progress bar overlay.
 *
 * DEBUG: Static bar first to test if canvas renders at all.
 * Then we'll hook up the event listener.
 */

import { listen } from "@tauri-apps/api/event";

interface OverlayPayload {
  progress: number;
  color: string;
}

const canvas = document.getElementById("progress-canvas") as HTMLCanvasElement;
const ctx = canvas.getContext("2d")!;

// Setup canvas
canvas.height = 4;
canvas.width = window.innerWidth || 1920;

let currentProgress = 0.35; // TEST: Start at 35% so we see something
let currentColor = "#ffc107"; // TEST: Amber color
let targetProgress = 0.35;

console.log("🎨 Canvas setup: " + canvas.width + "x" + canvas.height);

function animate() {
  currentProgress += (targetProgress - currentProgress) * 0.1;

  // Draw background
  ctx.fillStyle = "#000000";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  // Draw bar
  const barWidth = currentProgress * canvas.width;
  ctx.fillStyle = currentColor;
  ctx.fillRect(0, 0, barWidth, canvas.height);

  console.log(`🎬 Drawing: ${Math.round(currentProgress * 100)}% | ${currentColor}`);

  requestAnimationFrame(animate);
}

// Listen for events with delay
setTimeout(() => {
  console.log("📡 Attaching event listener after 500ms delay...");
  listen<OverlayPayload>("overlay:progress", ({ payload }) => {
    console.log("✓✓✓ EVENT RECEIVED! ✓✓✓", payload);
    targetProgress = Math.min(payload.progress, 1.0);
    currentColor = payload.color;
  }).catch((err) => {
    console.error("✗ Listener error:", err);
  });
}, 500);

// Start animation
animate();
