# Morning checklist (2026-10-21) — status 2026-10-23

Everything below needs you (SSH access, Apple ID, Bluehost login or the phone cable).
Each item is independent; order is a suggestion.

## 1. Server on the Hetzner box — done

`https://wars.3m4.net` is live (deployed by the `deploy` workflow once the DEPLOY_*
secrets were set on 2026-10-22; 3m4 PR #15 merged). Verified on 2026-10-23 with
`cargo run -p wars-client --features net --example smoke -- https://wars.3m4.net`:
two guests, a lobby, an AI, 100 lockstep ticks with matching hashes. Still to confirm
on the box: the nightly backup cron line below.

```bash
ssh root@46.225.62.18
git clone https://github.com/Devace-93/wars /opt/wars && bash /opt/wars/deploy/first-deploy.sh
# then: merge https://github.com/Devace-93/3m4/pull/15 and reload Caddy
cd /opt/3m4 && git pull && docker compose -f deploy/docker-compose.yml -f deploy/docker-compose.prod.yml exec caddy caddy reload --config /etc/caddy/Caddyfile
# nightly backup
(crontab -l; echo "5 3 * * * /opt/wars/deploy/backup.sh") | crontab -
```

Check: `curl https://wars.3m4.net/healthz` → `{"status":"ok"}` (after DNS below).

## 2. DNS at Bluehost — done

`wars.3m4.net` A → `46.225.62.18` (TTL 15 min) was added on 2026-10-21 and already
answers from ns1/ns2.bluehost.com and Google DNS. Nothing to do.

## 3. Deploy on push — done (secrets + DEPLOY_ENABLED set 2026-10-22)

Repository settings → Secrets: `DEPLOY_HOST` = `46.225.62.18`, `DEPLOY_USER` = `root`,
`DEPLOY_KEY` = a private ed25519 key whose public half is in the server's
`authorized_keys`; Variables: `DEPLOY_ENABLED` = `true`. From then on every merge to
`main` redeploys through `.github/workflows/deploy.yml`.

## 4. Phone — done (installed and verified on the Xiaomi 2026-10-21)

Plug the Xiaomi back in with USB debugging on and tell me "install the APK" (or run
`make android` then `adb install -r apps/android/app/build/outputs/apk/debug/app-debug.apk`).
The APK was verified on an Android 14 emulator overnight (menus, setup, game screen).

## 5. iPhone (15 minutes, free Apple ID)

1. The `ios` workflow on GitHub Actions builds `wars-ios.ipa` (unsigned) on every push
   to `main` that touches the code; download the artifact from the latest run
   (Actions → ios → Artifacts). If the first run failed, open the log: the Swift shell
   was written without a Mac to test on, so a compile error there is possible and is a
   five-minute fix.
2. Install **Sideloader** (Linux: https://github.com/Dadoum/Sideloader/releases, AppImage)
   or Sideloadly on Windows. Plug in the iPhone, log in with a free Apple ID, pick the
   `.ipa`. Free accounts re-sign every 7 days.
3. On the iPhone: Settings → General → VPN & Device Management → trust the profile.

## 6. Info site — PR #14 now says `live`; merge it

Merge https://github.com/Devace-93/3m4-info/pull/14 (card status `building`); flip it to
`live` with `https://wars.3m4.net` once the server answers.

## 7. Play

- Desktop: `cargo run --release -p wars-desktop` (or `make desktop`). F12 takes a screenshot.
- Console: `cargo run --release -p wars-tui` (`--quick` skips the menu).
- Online: Multiplayer → guest or your 3m4 email + password → Create game → share the
  room code; the phone joins the same way. Online saves need an account, not a guest.

## Known gaps (documented, not hidden)

- GIF/webm PR demos: ffmpeg/xvfb are not installed (needs sudo), so PRs carry PNG
  screenshots instead.
- iOS was compiled only in CI, never run on a device.
- The AI does not use trebuchets; gates are walkable by everyone; no elevation gameplay.
  See `docs/DESIGN.md` → "Simplifications".
- Multiplayer was tested with two local clients and an AI through the real server (CI
  integration test); not yet across the internet.
