#!/usr/bin/env node
/**
 * google-fit-auth.cjs — one-shot helper to obtain a Google refresh_token
 * with the Fitness scope.
 *
 * Usage:
 *   1. Ensure apps/desk/.env contains GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET
 *   2. In Google Cloud Console → Credentials → your OAuth Client →
 *      add "http://localhost:8765/callback" to Authorized redirect URIs
 *   3. node apps/desk/scripts/google-fit-auth.cjs
 *   4. Browser opens → click Allow → script prints GOOGLE_REFRESH_TOKEN
 *   5. Paste that value into apps/desk/.env
 *
 * No npm deps — uses only Node built-ins.
 */
"use strict";

const http = require("node:http");
const fs = require("node:fs");
const path = require("node:path");
const { spawn } = require("node:child_process");

const PORT = 8765;
const REDIRECT_URI = `http://localhost:${PORT}/callback`;
const SCOPE = "https://www.googleapis.com/auth/fitness.activity.read";

function loadEnv(envPath) {
  if (!fs.existsSync(envPath)) {
    throw new Error(`.env not found at ${envPath}`);
  }
  const out = {};
  for (const line of fs.readFileSync(envPath, "utf8").split(/\r?\n/)) {
    const m = line.match(/^\s*([A-Z0-9_]+)\s*=\s*(.*)\s*$/);
    if (m) out[m[1]] = m[2].replace(/^["']|["']$/g, "");
  }
  return out;
}

/**
 * Open a URL in the user's default browser.
 * Uses spawn with array args (no shell) to avoid injection — the URL is
 * passed as a single argv element, never concatenated into a command string.
 */
function openBrowser(url) {
  let cmd, args;
  if (process.platform === "win32") {
    // rundll32 sidesteps cmd.exe — needed because `cmd /c start` mangles
    // URLs containing `&` (interprets them as command separators) even
    // when passed via spawn's argv array.
    cmd = "rundll32";
    args = ["url.dll,FileProtocolHandler", url];
  } else if (process.platform === "darwin") {
    cmd = "open";
    args = [url];
  } else {
    cmd = "xdg-open";
    args = [url];
  }
  const child = spawn(cmd, args, { stdio: "ignore", detached: true });
  child.on("error", () => {
    // Non-fatal: user can copy/paste the URL from the console.
  });
  child.unref();
}

function buildConsentUrl(clientId) {
  const params = new URLSearchParams({
    client_id: clientId,
    redirect_uri: REDIRECT_URI,
    response_type: "code",
    scope: SCOPE,
    access_type: "offline",
    prompt: "consent",
  });
  return `https://accounts.google.com/o/oauth2/v2/auth?${params}`;
}

async function exchangeCode(clientId, clientSecret, code) {
  const body = new URLSearchParams({
    code,
    client_id: clientId,
    client_secret: clientSecret,
    redirect_uri: REDIRECT_URI,
    grant_type: "authorization_code",
  });
  const resp = await fetch("https://oauth2.googleapis.com/token", {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body,
  });
  if (!resp.ok) {
    throw new Error(`token exchange http ${resp.status}: ${await resp.text()}`);
  }
  return resp.json();
}

/**
 * Escape user-controlled text before inserting into the HTML response.
 * Threat model is "I'm clicking my own consent" — XSS impact is near zero,
 * but the `err` param comes off the URL and reflecting it raw is sloppy.
 */
function escapeHtml(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function waitForCode() {
  return new Promise((resolve, reject) => {
    const server = http.createServer((req, res) => {
      const url = new URL(req.url, `http://localhost:${PORT}`);
      if (url.pathname !== "/callback") {
        res.writeHead(404).end("not found");
        return;
      }
      const code = url.searchParams.get("code");
      const err = url.searchParams.get("error");
      if (err) {
        res
          .writeHead(400, { "Content-Type": "text/html; charset=utf-8" })
          .end(
            `<h1>OAuth error</h1><pre>${escapeHtml(err)}</pre><p>Close this tab and re-run the script.</p>`,
          );
        server.close();
        reject(new Error(`oauth error: ${err}`));
        return;
      }
      if (!code) {
        res.writeHead(400).end("missing code");
        return;
      }
      res
        .writeHead(200, { "Content-Type": "text/html; charset=utf-8" })
        .end(
          `<h1>OK</h1><p>You can close this tab and return to the terminal.</p>`,
        );
      server.close();
      resolve(code);
    });
    server.on("error", reject);
    server.listen(PORT);
  });
}

async function main() {
  const envPath = path.resolve(__dirname, "..", ".env");
  const env = loadEnv(envPath);
  const clientId = env.GOOGLE_CLIENT_ID;
  const clientSecret = env.GOOGLE_CLIENT_SECRET;
  if (!clientId || !clientSecret) {
    throw new Error(
      "GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET must be set in apps/desk/.env",
    );
  }

  const consent = buildConsentUrl(clientId);
  console.log("Opening browser for consent…");
  console.log("If it does not open, visit:\n  " + consent + "\n");
  openBrowser(consent);

  const code = await waitForCode();
  console.log("Got authorization code; exchanging for tokens…");
  const tokens = await exchangeCode(clientId, clientSecret, code);

  if (!tokens.refresh_token) {
    console.error(
      "\nGoogle did not return a refresh_token. This usually means you have\n" +
        "previously consented and Google is suppressing the refresh_token.\n" +
        "Revoke the app at https://myaccount.google.com/permissions and retry.",
    );
    process.exit(2);
  }

  console.log("\n=== SUCCESS ===");
  console.log("Add the following line to apps/desk/.env:\n");
  console.log(`GOOGLE_REFRESH_TOKEN=${tokens.refresh_token}\n`);
  console.log(
    "(access_token shown for sanity:",
    tokens.access_token.slice(0, 12) + "…)",
  );
}

main().catch((e) => {
  console.error("FAILED:", e.message);
  process.exit(1);
});
