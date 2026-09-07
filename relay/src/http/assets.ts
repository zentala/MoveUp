/**
 * assets.ts — `/app/*`, the viewer build.
 *
 * The same React bundle the desktop ships, served from the `ASSETS` binding
 * (`wrangler.toml [assets] directory = "../dist"`). The binding is absent
 * before a `vite build`, and that case answers 503 `assets_unavailable`
 * instead of 404: "the page is missing" and "the build is missing" are
 * different problems and must not look alike in a browser.
 */
import type { Env } from "../env";
import { fail } from "./responses";

export const APP_PREFIX = "/app";

export async function serveApp(request: Request, env: Env): Promise<Response> {
  if (!env.ASSETS) {
    return fail(
      "assets_unavailable",
      "the viewer build is not bound to this Worker — run `vite build` and redeploy",
      503,
    );
  }
  const url = new URL(request.url);
  // The bundle is built for the site root; strip the mount point before asking
  // the asset store, and fall back to the SPA entry so `#/pair` deep links work.
  const stripped = url.pathname.slice(APP_PREFIX.length) || "/";
  return env.ASSETS.fetch(new Request(new URL(stripped, url.origin), request));
}
