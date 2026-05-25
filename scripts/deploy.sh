#!/usr/bin/env bash
# Pull-based автодеплой FluxRadar (сервер за NAT — забираем сами, без входящих).
# Запускается по таймеру: если origin/main впереди — pull, пересборка изменённого,
# перезапуск. Идемпотентен: нет новых коммитов → ничего не делает.
#
# Лог: /tmp/flux-deploy.log. Запуск таймером (cron/systemd) от пользователя nel.
set -uo pipefail

ROOT="${ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

# Самокопирование: git reset/pull ниже может перезаписать ЭТОТ файл на ходу
# (bash читает скрипт построчно). Копируем себя в /tmp и работаем оттуда.
if [ "${FLUX_DEPLOY_DETACHED:-}" != "1" ]; then
  cp "${BASH_SOURCE[0]}" /tmp/flux-deploy-running.sh 2>/dev/null || exit 1
  chmod +x /tmp/flux-deploy-running.sh
  FLUX_DEPLOY_DETACHED=1 ROOT="$ROOT" exec bash /tmp/flux-deploy-running.sh
fi

cd "$ROOT" || exit 1
LOG=/tmp/flux-deploy.log
log() { echo "$(date '+%F %T') $*" >> "$LOG"; }

# Окружение сборки (rustup + nvm).
source "$HOME/.cargo/env" 2>/dev/null || true
export PATH="$HOME/.nvm/versions/node/v20.20.1/bin:$PATH"

# Лок: не запускать второй деплой параллельно.
exec 9>/tmp/flux-deploy.lock
flock -n 9 || { log "уже идёт деплой — пропуск"; exit 0; }

git fetch origin main --quiet 2>>"$LOG" || { log "git fetch не удался"; exit 1; }
LOCAL=$(git rev-parse HEAD)
REMOTE=$(git rev-parse origin/main)
if [ "$LOCAL" = "$REMOTE" ]; then
  exit 0   # нет изменений — тихо выходим
fi

log "новые коммиты $LOCAL → $REMOTE, деплой"
# Что изменилось между текущим и удалённым — чтобы пересобрать только нужное.
CHANGED=$(git diff --name-only "$LOCAL" "$REMOTE" 2>/dev/null)
git reset --hard origin/main --quiet 2>>"$LOG" || { log "reset не удался"; exit 1; }

WEB_CHANGED=$(echo "$CHANGED" | grep -c '^web/' || true)
RUST_CHANGED=$(echo "$CHANGED" | grep -cE '^crates/|^Cargo\.' || true)

# --- Фронт: пересобрать dist (контейнер Nginx монтирует web/dist, подхватит сразу). ---
if [ "$WEB_CHANGED" -gt 0 ]; then
  log "пересборка web ($WEB_CHANGED файлов)"
  ( cd web && npm install --no-audit --no-fund >>"$LOG" 2>&1 && npm run build >>"$LOG" 2>&1 ) \
    && log "web собран" || log "ОШИБКА сборки web"
fi

# --- Rust: пересобрать release и перезапустить нужные бинари. ---
if [ "$RUST_CHANGED" -gt 0 ]; then
  log "пересборка rust ($RUST_CHANGED файлов)"
  set -a; . ./.env 2>/dev/null; set +a
  if cargo build --release -p api -p bot -p worker >>"$LOG" 2>&1; then
    log "rust собран, перезапуск сервисов"
    restart() { # $1 = имя бинаря, pidfile
      local name="$1" pidf="/tmp/flux-$1.pid"
      [ -f "$pidf" ] && kill "$(cat "$pidf")" 2>/dev/null
      sleep 1
      setsid bash -c "./target/release/$name > /tmp/flux-$name-prod.log 2>&1 & echo \$! > $pidf"
    }
    # api слушает порт — освободим явно
    OLD=$(ss -ltnp 2>/dev/null | grep ':5049' | grep -oE 'pid=[0-9]+' | head -1 | cut -d= -f2)
    [ -n "${OLD:-}" ] && kill "$OLD" 2>/dev/null
    restart api; restart bot; restart worker
    sleep 3
    log "сервисы перезапущены"
  else
    log "ОШИБКА сборки rust — сервисы не трогаю"
  fi
fi

# Сброс кэша summary, чтобы фронт сразу увидел новые расчёты.
docker exec fluxscope_redis redis-cli --scan --pattern 'summary:*' 2>/dev/null \
  | xargs -r docker exec fluxscope_redis redis-cli del >/dev/null 2>&1 || true

log "деплой завершён ($REMOTE)"
