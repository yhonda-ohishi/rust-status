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

// "11/08 01:23" JST — login moment = fetch time minus connected duration.
function fmtLogin(epoch: number) {
  const parts = Object.fromEntries(
    new Intl.DateTimeFormat('ja-JP', {
      timeZone: 'Asia/Tokyo',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      hour12: false,
    })
      .formatToParts(new Date(epoch * 1000))
      .map((p) => [p.type, p.value]),
  )
  return `${parts.month}/${parts.day} ${parts.hour}:${parts.minute}`
}

interface Row { name: string; duration_secs: number; login: number }

const rows = computed<Row[]>(() =>
  status.value
    ? status.value.player_list
        .map((p) => ({ ...p, login: Math.round(status.value!.fetched_at - p.duration_secs) }))
        .sort((a, b) => a.login - b.login)
    : [],
)

// The selected player survives reloads via localStorage. Names are per-connection
// pseudonyms, so identify by name + login time (login drifts a few seconds between
// queries), falling back to login time alone.
const PICK_KEY = 'rust-status:pick'
const TOLERANCE = 15
const pick = ref<{ name: string; login: number } | null>(null)

onMounted(() => {
  try {
    const raw = localStorage.getItem(PICK_KEY)
    if (raw) pick.value = JSON.parse(raw)
  } catch {}
})

const pickedIndex = computed(() => {
  const p = pick.value
  if (!p) return -1
  let best = -1
  let bestScore = Infinity
  rows.value.forEach((r, i) => {
    const diff = Math.abs(r.login - p.login)
    if (diff > TOLERANCE) return
    const score = diff - (r.name === p.name ? 1000 : 0)
    if (score < bestScore) { bestScore = score; best = i }
  })
  return best
})

function toggle(i: number) {
  const r = rows.value[i]
  pick.value = pickedIndex.value === i ? null : { name: r.name, login: r.login }
  try {
    if (pick.value) localStorage.setItem(PICK_KEY, JSON.stringify(pick.value))
    else localStorage.removeItem(PICK_KEY)
  } catch {}
}

// "03:39:31" — always 2-digit, JST.
const fetchedAt = computed(() =>
  status.value
    ? new Date(status.value.fetched_at * 1000).toLocaleTimeString('ja-JP', {
        timeZone: 'Asia/Tokyo',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        hour12: false,
      })
    : '',
)

// Ticks every second so "◯秒前" stays live without re-querying the server.
const now = ref(Date.now())
let timer: ReturnType<typeof setInterval> | undefined
onMounted(() => {
  load()
  timer = setInterval(() => (now.value = Date.now()), 1000)
})
onUnmounted(() => clearInterval(timer))

const ago = computed(() => {
  if (!status.value) return ''
  const s = Math.max(0, Math.floor(now.value / 1000 - status.value.fetched_at))
  if (s < 60) return `${s}秒前`
  if (s < 3600) return `${Math.floor(s / 60)}分前`
  return `${Math.floor(s / 3600)}時間前`
})
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
      <p class="fetched">
        <span class="label">取得</span>
        <span class="time">{{ fetchedAt }}</span>
        <span class="ago">{{ ago }}</span>
      </p>
      <p class="meta">map: {{ status.map }}</p>

      <h2>接続中 ({{ status.player_list.length }})</h2>
      <p class="note">名前はサーバーが返す仮名で、接続のたびに変わります。右はログイン日時。行をタップでハイライト。</p>
      <ul>
        <li
          v-for="(p, i) in rows"
          :key="i"
          :class="{ picked: i === pickedIndex }"
          @click="toggle(i)"
        >
          <span>{{ p.name || '(名前なし)' }}</span>
          <span class="dur" :title="fmtDuration(p.duration_secs)">{{ fmtLogin(p.login) }}</span>
        </li>
      </ul>
    </section>
  </main>
</template>

<style>
:root {
  --bg: #f6f5f2; --fg: #1d1d1b; --muted: #6b6a66; --card: #ffffff; --line: #e2e0da;
  --accent: #c4512b; --err: #b42318; --time: #1d5fd1; --pick: #fde9a8;
  color-scheme: light dark;
}
@media (prefers-color-scheme: dark) {
  :root { --bg: #161615; --fg: #ecebe7; --muted: #9b9a95; --card: #20201e; --line: #34332f; --accent: #e07a52; --err: #f97066; --time: #6ea8ff; --pick: #4a3f12; }
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
.fetched { display: flex; align-items: baseline; gap: 10px; margin: 8px 0 2px; }
.fetched .label { color: var(--muted); font-size: .85rem; }
.fetched .time { color: var(--time); font-size: 1.25rem; font-weight: 600; font-variant-numeric: tabular-nums; letter-spacing: .02em; }
.fetched .ago { color: var(--muted); font-size: .85rem; font-variant-numeric: tabular-nums; }
.meta, .note { color: var(--muted); font-size: .85rem; margin: 4px 0; }
h2 { font-size: .95rem; margin: 20px 0 4px; }
ul { list-style: none; padding: 0; margin: 8px 0 0; }
li { display: flex; justify-content: space-between; padding: 8px 10px; margin: 0 -10px; border-top: 1px solid var(--line); cursor: pointer; border-radius: 6px; }
li.picked { background: var(--pick); font-weight: 600; }
li.picked .dur { color: var(--fg); }
.dur { color: var(--muted); font-variant-numeric: tabular-nums; }
.error { color: var(--err); margin-top: 16px; }
</style>
