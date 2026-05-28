// Персист выбранного адреса кошелька: URL query ?wallet= ↔ localStorage.
// Приоритет загрузки: URL ?wallet= → localStorage → fallback (DEFAULT_ADDRESS).

const STORAGE_KEY = 'fluxscope:wallet'
const HISTORY_KEY = 'fluxscope:wallet-history'
const HISTORY_LIMIT = 8
const PARAM = 'wallet'

/** Прочитать стартовый адрес: ?wallet= > localStorage > fallback. */
export function readWallet(fallback: string): string {
  try {
    const fromUrl = new URLSearchParams(window.location.search).get(PARAM)?.trim()
    if (fromUrl) return fromUrl
    const fromStorage = localStorage.getItem(STORAGE_KEY)?.trim()
    if (fromStorage) return fromStorage
  } catch {
    // приватный режим / недоступный storage — отдаём fallback
  }
  return fallback
}

/** Записать адрес в localStorage и в URL (?wallet=) без перезагрузки страницы. */
export function writeWallet(address: string): void {
  try {
    localStorage.setItem(STORAGE_KEY, address)
  } catch {
    // storage недоступен — игнорируем, URL всё равно обновим
  }
  const url = new URL(window.location.href)
  url.searchParams.set(PARAM, address)
  window.history.replaceState(null, '', url)
}

/** Текущий адрес из ?wallet= (для обработки кнопки «назад»), либо null. */
export function walletFromUrl(): string | null {
  const v = new URLSearchParams(window.location.search).get(PARAM)?.trim()
  return v || null
}

// --- История прошлых кошельков (для автодополнения в строке поиска). ---

/** Список прошлых адресов (свежие сверху). */
export function getWalletHistory(): string[] {
  try {
    const raw = localStorage.getItem(HISTORY_KEY)
    if (!raw) return []
    const arr = JSON.parse(raw)
    return Array.isArray(arr) ? arr.filter((x): x is string => typeof x === 'string') : []
  } catch {
    return []
  }
}

/** Добавить адрес в историю (в начало, дедуп без учёта регистра, лимит). */
export function addWalletHistory(address: string): void {
  const addr = address.trim()
  if (!addr) return
  try {
    const prev = getWalletHistory().filter((a) => a.toLowerCase() !== addr.toLowerCase())
    const next = [addr, ...prev].slice(0, HISTORY_LIMIT)
    localStorage.setItem(HISTORY_KEY, JSON.stringify(next))
  } catch {
    // storage недоступен — игнорируем
  }
}

/** Удалить конкретный адрес из истории. */
export function removeWalletHistory(address: string): void {
  try {
    const next = getWalletHistory().filter((a) => a.toLowerCase() !== address.toLowerCase())
    localStorage.setItem(HISTORY_KEY, JSON.stringify(next))
  } catch {
    // storage недоступен — игнорируем
  }
}
