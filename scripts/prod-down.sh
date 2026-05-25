#!/usr/bin/env bash
# Остановить прод-стек FluxScope. По умолчанию PG/Redis с данными НЕ трогаем.
#   --all  также останавливает PG + Redis (данные в volume сохраняются).
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "==> Останавливаю Nginx"
docker rm -f fluxscope_web >/dev/null 2>&1 && echo "   Nginx остановлен" || echo "   Nginx не был запущен"

echo "==> Останавливаю release-api"
if [[ -f /tmp/flux-api.pid ]] && kill -0 "$(cat /tmp/flux-api.pid)" 2>/dev/null; then
  kill "$(cat /tmp/flux-api.pid)" 2>/dev/null && echo "   api остановлен"
  rm -f /tmp/flux-api.pid
else
  echo "   api не был запущен"
fi

if [[ "${1:-}" == "--all" ]]; then
  echo "==> Останавливаю PG + Redis (данные в volume сохраняются)"
  docker-compose -f infra/docker-compose.yml stop || true
fi

echo "✅ Готово."
