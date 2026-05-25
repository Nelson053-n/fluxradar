// Форматирование чисел/адресов. Числа — всегда в JetBrains Mono (см. компоненты).

/** Заглушка для значений, которых наш API пока не отдаёт. */
export const NA = '—'

/** Целое с разделителями тысяч: 2695555 → "2,695,555". */
export function formatInt(n: number): string {
  return Math.round(n).toLocaleString('en-US')
}

/** Число с фиксированными дробными и разделителями: 198692.05 → "198,692.05". */
export function formatNum(n: number, fractionDigits = 2): string {
  return n.toLocaleString('en-US', {
    minimumFractionDigits: fractionDigits,
    maximumFractionDigits: fractionDigits,
  })
}

/**
 * Сумма FLUX «по-умному»: мелкие значения (< 1000) — с дробной частью (1 знак),
 * крупные — целыми с разделителями. Для daily/мелких доходов: «71.3 FLUX».
 */
export function formatFlux(n: number): string {
  if (n !== 0 && Math.abs(n) < 1000) return formatNum(n, 1)
  return formatInt(n)
}

/**
 * USD «по-умному»: мелкие суммы (< $100) — с центами («$5.12»),
 * крупные — целыми долларами («$199,009»).
 */
export function formatUsdSmart(n: number): string {
  if (n !== 0 && Math.abs(n) < 100) return formatUsd(n, 2)
  return formatUsd(n, 0)
}

/** USD: 199008.8 → "$199,009". */
export function formatUsd(n: number, fractionDigits = 0): string {
  return (
    '$' +
    n.toLocaleString('en-US', {
      minimumFractionDigits: fractionDigits,
      maximumFractionDigits: fractionDigits,
    })
  )
}

/** Цена FLUX с точностью до 4 знаков: 0.073711 → "$0.0737". */
export function formatPrice(n: number): string {
  return '$' + n.toFixed(4)
}

/** Сокращение адреса по центру: t1JSp9NE…UNHDrgyX. */
export function shortAddress(addr: string, head = 8, tail = 8): string {
  if (addr.length <= head + tail + 1) return addr
  return `${addr.slice(0, head)}···${addr.slice(-tail)}`
}

const MINUTE = 60
const HOUR = 60 * MINUTE
const DAY = 24 * HOUR
const MONTH = 30 * DAY
const YEAR = 365 * DAY

/** Язык для локализации единиц времени (совпадает с i18n Lang). */
export type TimeLang = 'en' | 'ru' | 'zh'

/** Локализованные суффиксы единиц времени: год / месяц / день / час / минута / секунда. */
const TIME_UNITS: Record<TimeLang, { y: string; mo: string; d: string; h: string; m: string; s: string }> = {
  en: { y: 'y', mo: 'mo', d: 'd', h: 'h', m: 'm', s: 's' },
  ru: { y: 'г', mo: 'мес', d: 'д', h: 'ч', m: 'мин', s: 'с' },
  zh: { y: '年', mo: '月', d: '天', h: '时', m: '分', s: '秒' },
}

/** «через ~» / «~ ago» — локализованная обёртка для относительного времени. */
const AGO: Record<TimeLang, string> = { en: 'ago', ru: 'назад', zh: '前' }

/**
 * Человекочитаемый возраст из секунд: «2y 3mo» / «2г 3мес» / «2年3月».
 * Берём две старшие ненулевые единицы для крупных значений, одну — для мелких.
 * Единицы локализуются по lang (по умолчанию en).
 */
export function formatAge(secs: number, lang: TimeLang = 'en'): string {
  if (!Number.isFinite(secs) || secs < 0) return NA
  const u = TIME_UNITS[lang]
  if (secs < MINUTE) return `${Math.floor(secs)}${u.s}`
  if (secs < HOUR) return `${Math.floor(secs / MINUTE)}${u.m}`
  if (secs < DAY) {
    const h = Math.floor(secs / HOUR)
    const m = Math.floor((secs % HOUR) / MINUTE)
    return m ? `${h}${u.h} ${m}${u.m}` : `${h}${u.h}`
  }
  if (secs < MONTH) return `${Math.floor(secs / DAY)}${u.d}`
  if (secs < YEAR) {
    const mo = Math.floor(secs / MONTH)
    const d = Math.floor((secs % MONTH) / DAY)
    return d ? `${mo}${u.mo} ${d}${u.d}` : `${mo}${u.mo}`
  }
  const y = Math.floor(secs / YEAR)
  const mo = Math.floor((secs % YEAR) / MONTH)
  return mo ? `${y}${u.y} ${mo}${u.mo}` : `${y}${u.y}`
}

/**
 * Относительная давность от unix-timestamp (сек) до «сейчас»:
 * «3h ago» / «3ч назад» / «3时前». null → NA.
 */
export function formatRelative(unixSecs: number | null, lang: TimeLang = 'en'): string {
  if (unixSecs == null) return NA
  const diff = Date.now() / 1000 - unixSecs
  if (diff < 0) return formatAge(0, lang)
  const age = formatAge(diff, lang)
  // zh не использует пробел между значением и послелогом «前».
  return lang === 'zh' ? `${age}${AGO.zh}` : `${age} ${AGO[lang]}`
}

/**
 * Короткая длительность для payout/maintenance: «~1h 8m» / «~1ч 8м» / «~1时8分».
 * Грубее formatAge — режем по часам/минутам (для горизонтов до выплаты).
 * 0 или меньше минуты → null (вызывающий показывает «вот-вот»/«—»).
 */
export function formatDuration(secs: number, lang: TimeLang = 'en'): string {
  if (!Number.isFinite(secs) || secs < 0) return NA
  const u = TIME_UNITS[lang]
  const sep = lang === 'zh' ? '' : ' '
  if (secs < MINUTE) return `${Math.max(0, Math.floor(secs))}${u.s}`
  if (secs < HOUR) return `${Math.floor(secs / MINUTE)}${u.m}`
  if (secs < DAY) {
    const h = Math.floor(secs / HOUR)
    const m = Math.floor((secs % HOUR) / MINUTE)
    return m ? `${h}${u.h}${sep}${m}${u.m}` : `${h}${u.h}`
  }
  const d = Math.floor(secs / DAY)
  const h = Math.floor((secs % DAY) / HOUR)
  return h ? `${d}${u.d}${sep}${h}${u.h}` : `${d}${u.d}`
}

/** Знак изменения для выбора цвета (success / danger / нейтрально). */
export type PctTone = 'up' | 'down' | 'flat'

/** «+9.32%» / «−1.40%» / «0.00%» + тон для подсветки. Минус — типографский (−). */
export function formatPctChange(n: number): { text: string; tone: PctTone } {
  const tone: PctTone = n > 0 ? 'up' : n < 0 ? 'down' : 'flat'
  const sign = n > 0 ? '+' : n < 0 ? '−' : ''
  return { text: `${sign}${Math.abs(n).toFixed(2)}%`, tone }
}

/**
 * Эмодзи-флаг из ISO-3166 alpha-2 кода (regional indicator symbols).
 * Невалидный/пустой код → пустая строка.
 */
export function countryFlag(code: string | null | undefined): string {
  if (!code || code.length !== 2 || !/^[A-Za-z]{2}$/.test(code)) return ''
  const A = 0x1f1e6
  const up = code.toUpperCase()
  return String.fromCodePoint(A + up.charCodeAt(0) - 65, A + up.charCodeAt(1) - 65)
}

/** Тон статус-бейджа ноды (online/offline/warning) по строке статуса. */
export type StatusTone = 'online' | 'offline' | 'warning'

/** CONFIRMED/«в детерминированном списке» → online; явный offline → offline; иначе warning. */
export function statusTone(status: string | null | undefined): StatusTone {
  const s = (status ?? '').toUpperCase()
  if (s === 'CONFIRMED' || s === 'STARTED' || s === 'ONLINE') return 'online'
  if (s === 'OFFLINE' || s === 'EXPIRED' || s === 'DOS') return 'offline'
  return 'warning'
}
