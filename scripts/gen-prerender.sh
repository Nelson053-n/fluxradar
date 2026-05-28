#!/usr/bin/env bash
# Генерация prerender-снимка для ботов (dynamic rendering).
# Headless-браузер открывает свой же прод (localhost:8080) с дефолтным кошельком,
# ждёт полной загрузки (гайд + калькулятор) и сохраняет готовый HTML в
# web/dist/prerender.html. Отдаётся ботам через nginx (map $http_user_agent).
# Запускается вручную и по cron раз в сутки. При ошибке старый снимок не трогается.
set -uo pipefail

ROOT="${ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
OUT="$ROOT/web/dist/prerender.html"
TMP="$(mktemp /tmp/flux-prerender.XXXXXX.html)"
# Кошелёк для снимка — с разнообразным флотом (все 3 тира), чтобы в HTML для ботов
# попала богатая таблица нод. Отличается от дефолта приложения (он маленький — для скорости людям).
URL="${PRERENDER_URL:-http://localhost:8080/?wallet=t1Mbb821cmQcT4bWovFWViA6MhPNFRVBajj}"

export PATH="$HOME/.nvm/versions/node/v20.20.1/bin:$PATH"
# Запускаем node из web/ — там playwright в node_modules (ESM bare-import резолвится по CWD).
if [ ! -d "$ROOT/web/node_modules/playwright" ]; then
  echo "playwright не найден в web/node_modules — npm install -D playwright"; rm -f "$TMP"; exit 1
fi
cd "$ROOT/web" || { rm -f "$TMP"; exit 1; }

OUT_TMP="$TMP" PRERENDER_URL="$URL" node --input-type=module <<'NODE'
import { chromium } from 'playwright'
import { writeFileSync } from 'node:fs'

const url = process.env.PRERENDER_URL
const out = process.env.OUT_TMP
const browser = await chromium.launch()
try {
  const page = await browser.newPage({ viewport: { width: 1280, height: 2400 } })
  await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 30000 })
  // Дожидаемся полной загрузки: калькулятор виден всегда, гайд — после данных кошелька.
  await page.waitForSelector('#calculator', { timeout: 30000 })
  await page.waitForSelector('#guide', { timeout: 30000 }).catch(() => {})
  await page.waitForTimeout(2500) // дорисовка живых данных
  let html = await page.content()
  // Вырезаем module-бандл — боты JS не исполняют, нужен только текст.
  html = html.replace(/<script\s+type="module"[^>]*><\/script>/g, '')
  html = html.replace(/<script\s+type="module"[^>]*\bsrc="[^"]*"[^>]*><\/script>/g, '')
  writeFileSync(out, html, 'utf8')
  console.log('snapshot bytes:', html.length)
} finally {
  await browser.close()
}
NODE

rc=$?
if [ $rc -ne 0 ] || [ ! -s "$TMP" ]; then
  echo "генерация снимка не удалась (rc=$rc) — старый prerender.html не тронут"
  rm -f "$TMP"; exit 1
fi

mv "$TMP" "$OUT"
chmod 644 "$OUT"  # nginx-контейнер должен прочитать
echo "prerender обновлён: $OUT ($(date '+%F %T'))"
