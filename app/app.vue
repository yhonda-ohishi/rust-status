<script setup lang="ts">
interface Player { name: string; duration_secs: number }
interface Status {
  ok: boolean
  error: string | null
  name: string
  map: string
  players: number
  max_players: number
  bots: number
  player_list: Player[]
  fetched_at: number
}

const status = ref<Status | null>(null)
const failure = ref<string | null>(null)
const loading = ref(false)

async function load() {
  loading.value = true
  failure.value = null
  try {
    const res = await fetch('/api/status', { cache: 'no-store' })
    if (!res.ok) throw new Error(`HTTP ${res.status}`)
    const body = (await res.json()) as Status
    if (!body.ok) throw new Error(body.error ?? 'query failed')
    status.value = body
  } catch (e) {
    failure.value = e instanceof Error ? e.message : String(e)
  } finally {
    loading.value = false
  }
}

function fmtDuration(s: number) {
  const h = Math.floor(s / 3600)
  const m = Math.floor((s % 3600) / 60)
  return h > 0 ? `${h}時間${m}分` : `${m}分`
}

const fetchedAt = computed(() =>
  status.value
    ? new Date(status.value.fetched_at * 1000).toLocaleTimeString('ja-JP', { timeZone: 'Asia/Tokyo' })
    : '',
)

onMounted(load)
</script>

<template>
  <main>
    <header>
      <h1>Rust Server Status</h1>
      <button :disabled="loading" @click="load">{{ loading ? '取得中…' : '更新' }}</button>
    </header>

    <p v-if="failure" class="error">取得できませんでした: {{ failure }}</p>

    <section v-if="status" class="card">
      <p class="server">{{ status.name }}</p>
      <p class="count">
        <span class="now">{{ status.players }}</span>
        <span class="max">/ {{ status.max_players }}</span>
      </p>
      <p class="meta">map: {{ status.map }} ・ 取得 {{ fetchedAt }} JST</p>

      <h2>接続中 ({{ status.player_list.length }})</h2>
      <p class="note">名前はサーバーが返す仮名で、接続のたびに変わります。</p>
      <ul>
        <li v-for="(p, i) in [...status.player_list].sort((a, b) => b.duration_secs - a.duration_secs)" :key="i">
          <span>{{ p.name || '(名前なし)' }}</span>
          <span class="dur">{{ fmtDuration(p.duration_secs) }}</span>
        </li>
      </ul>
    </section>
  </main>
</template>

<style>
:root {
  --bg: #f6f5f2; --fg: #1d1d1b; --muted: #6b6a66; --card: #ffffff; --line: #e2e0da;
  --accent: #c4512b; --err: #b42318;
  color-scheme: light dark;
}
@media (prefers-color-scheme: dark) {
  :root { --bg: #161615; --fg: #ecebe7; --muted: #9b9a95; --card: #20201e; --line: #34332f; --accent: #e07a52; --err: #f97066; }
}
* { box-sizing: border-box; }
body { margin: 0; background: var(--bg); color: var(--fg); font-family: system-ui, sans-serif; }
main { max-width: 560px; margin: 0 auto; padding: 24px 16px; }
header { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
h1 { font-size: 1.1rem; margin: 0; }
button { font: inherit; padding: 8px 18px; border-radius: 8px; border: 1px solid var(--line); background: var(--card); color: var(--fg); cursor: pointer; }
button:disabled { opacity: .6; cursor: default; }
.card { margin-top: 16px; background: var(--card); border: 1px solid var(--line); border-radius: 12px; padding: 20px; }
.server { margin: 0; font-weight: 600; overflow-wrap: anywhere; }
.count { margin: 12px 0 4px; }
.now { font-size: 3rem; font-weight: 700; color: var(--accent); font-variant-numeric: tabular-nums; }
.max { font-size: 1.4rem; color: var(--muted); margin-left: 6px; }
.meta, .note { color: var(--muted); font-size: .85rem; margin: 4px 0; }
h2 { font-size: .95rem; margin: 20px 0 4px; }
ul { list-style: none; padding: 0; margin: 8px 0 0; }
li { display: flex; justify-content: space-between; padding: 8px 0; border-top: 1px solid var(--line); }
.dur { color: var(--muted); font-variant-numeric: tabular-nums; }
.error { color: var(--err); margin-top: 16px; }
</style>
