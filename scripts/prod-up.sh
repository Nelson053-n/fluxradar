#!/usr/bin/env bash
# Поднять FluxScope в прод-режиме локально:
#   PG + Redis (Docker) → release-api (нативно) → Nginx со статикой web/dist.
# Дашборд: http://localhost:8080
#
# Примечание: на этом хосте docker-compose v1 (1.29) ломает общую сеть при
# down одного из стеков, поэтому Nginx поднимаем напрямую через `docker run`,
# а PG/Redis — идемпотентным `up -d` (не пересоздаём, чтобы не ронять данные).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

source "$HOME/.cargo/env" 2>/dev/null || true
export PATH="$HOME/.nvm/versions/node/v20.20.1/bin:$PATH"

echo "==> 1/4 PG + Redis (если не подняты)"
docker-compose -f infra/docker-compose.yml --env-file .env up -d

echo "==> Миграции (идемпотентно; ошибки 'already exists' игнорируем)"
# ждём готовности PG
for _ in $(seq 1 20); do
  docker exec fluxscope_pg pg_isready -U fluxscope >/dev/null 2>&1 && break
  sleep 1
done
docker exec -i fluxscope_pg psql -U fluxscope -d fluxscope -v ON_ERROR_STOP=0 \
  < migrations/0001_init.sql >/dev/null 2>&1 || true

echo "==> 2/4 Сборка фронта (web/dist)"
( cd web && npm run build >/dev/null )

echo "==> 3/4 Release-api на :5049"
cargo build --release -p api
set -a; . ./.env; set +a
# гасим прошлый инстанс по PID-файлу, чужие процессы не трогаем
if [[ -f /tmp/flux-api.pid ]] && kill -0 "$(cat /tmp/flux-api.pid)" 2>/dev/null; then
  kill "$(cat /tmp/flux-api.pid)" 2>/dev/null || true
  sleep 1
fi
# setsid: api переживает завершение этого скрипта
setsid bash -c './target/release/api > /tmp/flux-api-prod.log 2>&1 & echo $! > /tmp/flux-api.pid'
for _ in $(seq 1 20); do
  curl -s -o /dev/null --max-time 2 http://127.0.0.1:5049/api/v1/health && break
  sleep 1
done

echo "==> 4/4 Nginx на :8080"
docker rm -f fluxscope_web >/dev/null 2>&1 || true
docker run -d --name fluxscope_web \
  -p 8080:80 \
  --add-host host.docker.internal:host-gateway \
  -v "$ROOT/web/dist:/usr/share/nginx/html:ro" \
  -v "$ROOT/infra/nginx.prod.conf:/etc/nginx/nginx.conf:ro" \
  --restart unless-stopped \
  nginx:1.27-alpine >/dev/null

echo
LAN_IP=$(ip -4 addr show 2>/dev/null | grep -oE 'inet 192\.168\.[0-9.]+' | head -1 | awk '{print $2}')
echo "✅ FluxRadar поднят:"
echo "   локально : http://localhost:8080"
[ -n "$LAN_IP" ] && echo "   по сети  : http://$LAN_IP:8080"
echo "   api health : $(curl -s --max-time 5 http://127.0.0.1:5049/api/v1/health)"
echo "   api ready  : $(curl -s --max-time 15 http://127.0.0.1:5049/api/v1/ready)"
echo "   логи api   : tail -f /tmp/flux-api-prod.log"
