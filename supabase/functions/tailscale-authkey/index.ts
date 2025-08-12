// Minimal Supabase Edge Function to mint a single-use Tailscale pre-auth key
// POST /tailscale-authkey
// Env: TAILSCALE_OAUTH_CLIENT_ID, TAILSCALE_OAUTH_CLIENT_SECRET, TAILNET_NAME
import { createClient } from "jsr:@supabase/supabase-js@2";

Deno.serve(async (req) => {
  if (req.method === "OPTIONS") return new Response(null, { status: 204 });
  if (req.method !== "POST") return new Response("Method Not Allowed", { status: 405 });

  const auth = req.headers.get("Authorization") || "";
  if (!auth.startsWith("Bearer ")) return new Response("Unauthorized", { status: 401 });

  const SUPABASE_URL = Deno.env.get("SUPABASE_URL")!;
  const SUPABASE_ANON_KEY = Deno.env.get("SUPABASE_ANON_KEY")!;
  const TS_CLIENT_ID = Deno.env.get("TAILSCALE_OAUTH_CLIENT_ID")!;
  const TS_CLIENT_SECRET = Deno.env.get("TAILSCALE_OAUTH_CLIENT_SECRET")!;
  const TAILNET = Deno.env.get("TAILNET_NAME")!;
  if (!TS_CLIENT_ID || !TS_CLIENT_SECRET || !TAILNET) return new Response("Server not configured", { status: 500 });

  // RLS-verified: require operators.verified = true
  const supabase = createClient(SUPABASE_URL, SUPABASE_ANON_KEY, { global: { headers: { Authorization: auth } } });
  const { data: op, error: opErr } = await supabase.from("operators").select("verified").single();
  if (opErr || !op) return new Response("Forbidden", { status: 403 });
  if (!op.verified) return new Response("Forbidden", { status: 403 });

  // OAuth token
  const basic = "Basic " + btoa(`${TS_CLIENT_ID}:${TS_CLIENT_SECRET}`);
  const tokRes = await fetch("https://api.tailscale.com/api/v2/oauth/token", {
    method: "POST",
    headers: { "Authorization": basic, "Content-Type": "application/x-www-form-urlencoded" },
    body: new URLSearchParams({ grant_type: "client_credentials" })
  });
  if (!tokRes.ok) return new Response("OAuth failed", { status: 502 });
  const tok = await tokRes.json();
  const at = tok.access_token as string;

  // Create preauth key
  const caps = {
    capabilities: { devices: { create: { reusable: false, ephemeral: false, preauthorized: true, tags: ["tag:operator"] } } },
    expirySeconds: 300
  };
  const keyRes = await fetch(`https://api.tailscale.com/api/v2/tailnet/${encodeURIComponent(TAILNET)}/keys`, {
    method: "POST",
    headers: { "Authorization": `Bearer ${at}`, "Content-Type": "application/json" },
    body: JSON.stringify(caps)
  });
  if (!keyRes.ok) return new Response("Key create failed", { status: 502 });
  const body = await keyRes.json();
  const auth_key = body.key || body.authKey;
  const expires_at = body.expires || body.expiry || null;
  if (!auth_key) return new Response("Key missing", { status: 502 });

  return new Response(JSON.stringify({ auth_key, expires_at }), {
    status: 200,
    headers: { "Content-Type": "application/json", "Cache-Control": "no-store" }
  });
});


