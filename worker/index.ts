// /api/status -> A2S relay on mini-ryzen via Workers VPC. Everything else -> static assets.
interface Env {
  ASSETS: Fetcher
  RELAY: Fetcher
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url)
    if (url.pathname === '/api/status') {
      try {
        // Host/port come from the VPC Service config; only the path matters here.
        const res = await env.RELAY.fetch('http://relay/status')
        return new Response(res.body, {
          status: res.status,
          headers: { 'content-type': 'application/json', 'cache-control': 'no-store' },
        })
      } catch (e) {
        return Response.json(
          { ok: false, error: `relay unreachable: ${e instanceof Error ? e.message : String(e)}` },
          { status: 502, headers: { 'cache-control': 'no-store' } },
        )
      }
    }
    return env.ASSETS.fetch(request)
  },
} satisfies ExportedHandler<Env>
