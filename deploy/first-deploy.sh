#!/usr/bin/env bash
# First production deploy of wars on the 3m4 box. Run ON THE SERVER as root:
#   git clone https://github.com/Devace-93/wars /opt/wars && bash /opt/wars/deploy/first-deploy.sh
# or from your machine:
#   ssh root@<server> 'bash -s' < deploy/first-deploy.sh
#
# Idempotent: re-running keeps an existing .env and just rebuilds/restarts.
# Prerequisites (already true on the 3m4 box): `edge` docker network exists and
# 3m4's Caddyfile.prod has the wars.3m4.net block (PR deploy/wars-edge-routing).
set -euo pipefail

if [ ! -d /opt/wars ]; then
  git clone -q https://github.com/Devace-93/wars /opt/wars
fi
cd /opt/wars
git pull -q --ff-only

if [ ! -f .env ]; then
  cp .env.example .env
  set_var() { sed -i "s|^$1=.*|$1=$2|" .env; grep -q "^$1=" .env || echo "$1=$2" >> .env; }
  set_var ACCOUNTS_URL https://accounts.3m4.net
  set_var ACCOUNTS_INTERNAL_URL http://accounts-api-1:5000
  set_var GUEST_SIGNING_KEY "$(python3 -c 'import secrets; print(secrets.token_urlsafe(48))')"
  set_var PUBLIC_URL https://wars.3m4.net
  set_var RUST_LOG info
  chmod 600 .env
  echo "✓ .env written"
fi

cd deploy
docker compose --env-file ../.env --project-name wars -f docker-compose.yml -f docker-compose.prod.yml up -d --build --remove-orphans
docker image prune -f >/dev/null

echo "--- status"
docker compose --env-file ../.env --project-name wars -f docker-compose.yml -f docker-compose.prod.yml ps --format '{{.Name}} {{.Status}}'
docker run --rm --network edge curlimages/curl -s http://wars-server-1:8086/healthz
echo
echo "Now merge the 3m4 PR 'deploy/wars-edge-routing' and reload Caddy:"
echo "  cd /opt/3m4 && git pull && docker compose -f deploy/docker-compose.yml -f deploy/docker-compose.prod.yml exec caddy caddy reload --config /etc/caddy/Caddyfile"
