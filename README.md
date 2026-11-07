# rust-status

Rust ゲームサーバーの人数を A2S (UDP) で問い合わせて表示するページ。

```
Browser ─▶ Worker (static assets + /api/status) ─VPC─▶ cloudflared ─▶ a2s-relay ─UDP─▶ game server
```

- **ページ**: Nuxt (`ssr: false`) を `nuxt generate` し、Worker の static assets で配信。Cloudflare Access で保護。
- **Worker** (`worker/index.ts`): `/api/status` だけ Workers VPC 経由で中継へ。Workers は UDP を送れないため。
- **中継** (`relay/`): systemd のソケット起動 (`Accept=yes`)。接続ごとに 1 回だけ A2S_INFO + A2S_PLAYER を送って JSON を返し終了する。キャッシュなし。

A2S_PLAYER の名前は Rust サーバーが返す仮名で、接続のたびに変わる。

## deploy

```bash
npm install
npm run deploy
```

## 中継のインストール (中継マシン)

```bash
cd relay && cargo build --release
install -m755 target/release/relay ~/.local/bin/a2s-relay
cp systemd/a2s-relay.socket systemd/a2s-relay@.service ~/.config/systemd/user/
systemctl --user daemon-reload && systemctl --user enable --now a2s-relay.socket
```
