# Waitlist Worker — Specification

Cloudflare Worker for collecting waitlist signups from the desk.zentala.io landing page.

## Endpoint

```
POST https://waitlist.desk.zentala.io/api/signup
```

## Request

```json
{ "email": "user@example.com" }
```

## Response

| Status | Body | Meaning |
|--------|------|---------|
| 200 | `{ "ok": true }` | Signup recorded |
| 400 | `{ "error": "Invalid email" }` | Malformed or missing email |
| 409 | `{ "error": "Already registered" }` | Duplicate email |
| 429 | `{ "error": "Too many requests" }` | Rate limited (10 req/min per IP) |
| 405 | `{ "error": "Method not allowed" }` | Non-POST request to /api/signup |

## Storage — Cloudflare D1

```sql
CREATE TABLE waitlist (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  email TEXT UNIQUE NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  ip_hash TEXT NOT NULL,
  source TEXT DEFAULT 'website',
  tier_interest TEXT DEFAULT NULL,
  converted BOOLEAN DEFAULT FALSE
);

CREATE INDEX idx_waitlist_email ON waitlist(email);
CREATE INDEX idx_waitlist_created ON waitlist(created_at);
```

- `ip_hash`: SHA-256 of IP (privacy-preserving, useful for abuse detection)
- `tier_interest`: optional, tracks which tier the user clicked from (basic/pro/founder)
- `converted`: flipped when user completes a purchase (future)

## Rate Limiting — KV-based

Uses Cloudflare KV to track request counts per IP:
- Key: `rate:{ip_hash}` with 60s TTL
- Limit: 10 requests per minute per IP
- KV is eventually consistent, which is fine for rate limiting

---

## Full Implementation

### Project structure

```
waitlist-worker/
  wrangler.toml
  src/
    index.ts
  package.json
  tsconfig.json
```

### `wrangler.toml`

```toml
name = "desk-waitlist"
main = "src/index.ts"
compatibility_date = "2024-12-01"

[vars]
ALLOWED_ORIGIN = "https://desk.zentala.io"
RATE_LIMIT_MAX = "10"
RATE_LIMIT_WINDOW_SEC = "60"

[[d1_databases]]
binding = "DB"
database_name = "desk-waitlist"
database_id = "<generated-after-create>"

[[kv_namespaces]]
binding = "RATE_LIMIT"
id = "<generated-after-create>"

# Optional: Resend API key for welcome emails
# [vars]
# RESEND_API_KEY = "<set via wrangler secret>"
```

### `package.json`

```json
{
  "name": "desk-waitlist-worker",
  "version": "1.0.0",
  "private": true,
  "scripts": {
    "dev": "wrangler dev",
    "deploy": "wrangler deploy",
    "db:init": "wrangler d1 execute desk-waitlist --file=schema.sql",
    "db:init:local": "wrangler d1 execute desk-waitlist --local --file=schema.sql"
  },
  "devDependencies": {
    "@cloudflare/workers-types": "^4.20241205.0",
    "typescript": "^5.7.0",
    "wrangler": "^3.99.0"
  }
}
```

### `tsconfig.json`

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ES2022",
    "moduleResolution": "bundler",
    "lib": ["ES2022"],
    "types": ["@cloudflare/workers-types"],
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true
  },
  "include": ["src"]
}
```

### `schema.sql`

```sql
CREATE TABLE IF NOT EXISTS waitlist (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  email TEXT UNIQUE NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  ip_hash TEXT NOT NULL,
  source TEXT DEFAULT 'website',
  tier_interest TEXT DEFAULT NULL,
  converted BOOLEAN DEFAULT FALSE
);

CREATE INDEX IF NOT EXISTS idx_waitlist_email ON waitlist(email);
CREATE INDEX IF NOT EXISTS idx_waitlist_created ON waitlist(created_at);
```

### `src/index.ts`

```typescript
/**
 * Desk Waitlist Worker
 *
 * Collects email signups for desk.zentala.io pre-launch waitlist.
 * Stores in D1, rate-limits via KV, returns JSON responses.
 */

interface Env {
  DB: D1Database;
  RATE_LIMIT: KVNamespace;
  ALLOWED_ORIGIN: string;
  RATE_LIMIT_MAX: string;
  RATE_LIMIT_WINDOW_SEC: string;
  RESEND_API_KEY?: string;
}

interface SignupRequest {
  email: string;
  source?: string;
  tier_interest?: string;
}

const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
const VALID_TIERS = ['basic', 'pro', 'founder'];
const VALID_SOURCES = ['website', 'reddit', 'hackernews', 'twitter', 'referral'];

/** SHA-256 hash of a string, returned as hex. */
async function sha256(input: string): Promise<string> {
  const data = new TextEncoder().encode(input);
  const hash = await crypto.subtle.digest('SHA-256', data);
  return Array.from(new Uint8Array(hash))
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

/** Build a JSON response with CORS headers. */
function jsonResponse(
  body: Record<string, unknown>,
  status: number,
  origin: string
): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: {
      'Content-Type': 'application/json',
      'Access-Control-Allow-Origin': origin,
      'Access-Control-Allow-Methods': 'POST, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type',
    },
  });
}

/** Check and increment rate limit. Returns true if request is allowed. */
async function checkRateLimit(
  kv: KVNamespace,
  ipHash: string,
  max: number,
  windowSec: number
): Promise<boolean> {
  const key = `rate:${ipHash}`;
  const current = await kv.get(key);
  const count = current ? parseInt(current, 10) : 0;

  if (count >= max) {
    return false;
  }

  await kv.put(key, String(count + 1), { expirationTtl: windowSec });
  return true;
}

/** Send welcome email via Resend API (no-op if key not configured). */
async function sendWelcomeEmail(
  email: string,
  apiKey: string | undefined
): Promise<void> {
  if (!apiKey) {
    return;
  }

  try {
    await fetch('https://api.resend.com/emails', {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${apiKey}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        from: 'Desk by zentala <hello@desk.zentala.io>',
        to: [email],
        subject: "You're on the waitlist!",
        html: `
          <h2>Welcome to the Desk waitlist!</h2>
          <p>Thanks for signing up. We'll notify you when pre-orders open.</p>
          <p>In the meantime, check out what we're building:</p>
          <ul>
            <li>Open-source desktop app for sit/stand desk tracking</li>
            <li>ToF laser sensor that detects your desk height</li>
            <li>Smart coaching to help you move more</li>
          </ul>
          <p>— zentala</p>
        `,
      }),
    });
  } catch {
    // Welcome email is best-effort; don't fail the signup
  }
}

/** Handle POST /api/signup */
async function handleSignup(
  request: Request,
  env: Env,
  clientIp: string
): Promise<Response> {
  const origin = env.ALLOWED_ORIGIN;
  const ipHash = await sha256(clientIp);

  // Rate limit check
  const max = parseInt(env.RATE_LIMIT_MAX, 10) || 10;
  const window = parseInt(env.RATE_LIMIT_WINDOW_SEC, 10) || 60;
  const allowed = await checkRateLimit(env.RATE_LIMIT, ipHash, max, window);
  if (!allowed) {
    return jsonResponse({ error: 'Too many requests' }, 429, origin);
  }

  // Parse body
  let body: SignupRequest;
  try {
    body = await request.json();
  } catch {
    return jsonResponse({ error: 'Invalid JSON' }, 400, origin);
  }

  // Validate email
  const email = body.email?.trim().toLowerCase();
  if (!email || !EMAIL_REGEX.test(email)) {
    return jsonResponse({ error: 'Invalid email' }, 400, origin);
  }

  // Validate optional fields
  const source = VALID_SOURCES.includes(body.source ?? '')
    ? body.source
    : 'website';
  const tierInterest = VALID_TIERS.includes(body.tier_interest ?? '')
    ? body.tier_interest
    : null;

  // Insert into D1
  try {
    await env.DB.prepare(
      `INSERT INTO waitlist (email, ip_hash, source, tier_interest)
       VALUES (?, ?, ?, ?)`
    )
      .bind(email, ipHash, source, tierInterest)
      .run();
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err);
    if (message.includes('UNIQUE constraint failed')) {
      return jsonResponse({ error: 'Already registered' }, 409, origin);
    }
    console.error('D1 insert error:', message);
    return jsonResponse({ error: 'Internal error' }, 500, origin);
  }

  // Fire-and-forget welcome email
  sendWelcomeEmail(email, env.RESEND_API_KEY);

  return jsonResponse({ ok: true }, 200, origin);
}

/** Handle GET /api/stats (public count only). */
async function handleStats(env: Env): Promise<Response> {
  const origin = env.ALLOWED_ORIGIN;
  try {
    const result = await env.DB.prepare(
      'SELECT COUNT(*) as count FROM waitlist'
    ).first<{ count: number }>();
    return jsonResponse({ count: result?.count ?? 0 }, 200, origin);
  } catch {
    return jsonResponse({ error: 'Internal error' }, 500, origin);
  }
}

export default {
  async fetch(
    request: Request,
    env: Env,
    _ctx: ExecutionContext
  ): Promise<Response> {
    const url = new URL(request.url);
    const origin = env.ALLOWED_ORIGIN;

    // CORS preflight
    if (request.method === 'OPTIONS') {
      return new Response(null, {
        status: 204,
        headers: {
          'Access-Control-Allow-Origin': origin,
          'Access-Control-Allow-Methods': 'POST, GET, OPTIONS',
          'Access-Control-Allow-Headers': 'Content-Type',
          'Access-Control-Max-Age': '86400',
        },
      });
    }

    // Routes
    if (url.pathname === '/api/signup' && request.method === 'POST') {
      const clientIp = request.headers.get('CF-Connecting-IP') ?? '0.0.0.0';
      return handleSignup(request, env, clientIp);
    }

    if (url.pathname === '/api/stats' && request.method === 'GET') {
      return handleStats(env);
    }

    if (url.pathname === '/api/signup') {
      return jsonResponse({ error: 'Method not allowed' }, 405, origin);
    }

    return jsonResponse({ error: 'Not found' }, 404, origin);
  },
} satisfies ExportedHandler<Env>;
```

---

## Features

| Feature | Implementation |
|---------|---------------|
| Email validation | Regex check + trim + lowercase normalization |
| Duplicate detection | D1 UNIQUE constraint on email column |
| Rate limiting | KV-based, 10 req/min per IP (SHA-256 hashed) |
| CORS | Locked to `desk.zentala.io` origin |
| Welcome email | Resend API (optional, no-op if key not set) |
| Public stats | `GET /api/stats` returns signup count |
| Tier tracking | Optional `tier_interest` field (basic/pro/founder) |
| Source tracking | Optional `source` field (website/reddit/hackernews/twitter/referral) |
| Privacy | IP stored as SHA-256 hash, not raw |

## Deployment

### 1. Create infrastructure

```bash
cd waitlist-worker/

# Create D1 database
npx wrangler d1 create desk-waitlist
# Copy the database_id into wrangler.toml

# Create KV namespace for rate limiting
npx wrangler kv namespace create RATE_LIMIT
# Copy the id into wrangler.toml

# Initialize schema
npx wrangler d1 execute desk-waitlist --file=schema.sql
```

### 2. Configure secrets (optional)

```bash
# Welcome email via Resend (sign up at resend.com, free tier = 100 emails/day)
npx wrangler secret put RESEND_API_KEY
```

### 3. Deploy

```bash
npx wrangler deploy
```

### 4. DNS

In Cloudflare dashboard for `zentala.io`:

| Type | Name | Content | Proxy |
|------|------|---------|-------|
| CNAME | `waitlist.desk` | `desk-waitlist.<account>.workers.dev` | Proxied |

Or use a Worker Route on the `zentala.io` zone:
```
waitlist.desk.zentala.io/*  →  desk-waitlist
```

## Landing page integration

```typescript
async function submitWaitlist(
  email: string,
  tierInterest?: string
): Promise<{ ok: boolean; error?: string }> {
  const res = await fetch('https://waitlist.desk.zentala.io/api/signup', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, tier_interest: tierInterest }),
  });
  return res.json();
}
```

## Monitoring

- **Wrangler tail**: `npx wrangler tail` for live logs
- **D1 query**: `npx wrangler d1 execute desk-waitlist --command "SELECT COUNT(*) FROM waitlist"`
- **Signups by day**: `SELECT date(created_at) as day, COUNT(*) as signups FROM waitlist GROUP BY day ORDER BY day DESC`
- **Tier breakdown**: `SELECT tier_interest, COUNT(*) as count FROM waitlist GROUP BY tier_interest`
- **Source breakdown**: `SELECT source, COUNT(*) as count FROM waitlist GROUP BY source`

## Cost estimate

At pre-launch scale (< 10k signups):
- D1: free tier (5M rows read, 100k writes/day)
- KV: free tier (100k reads, 1k writes/day)
- Workers: free tier (100k requests/day)
- Resend: free tier (100 emails/day)
- **Total: $0/month** until significant traction
