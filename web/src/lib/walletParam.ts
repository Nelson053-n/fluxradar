// Персист выбранного адреса кошелька: URL query ?wallet= ↔ localStorage.
// Приоритет загрузки: URL ?wallet= → localStorage → fallback (DEFAULT_ADDRESS).

const STORAGE_KEY = 'fluxscope:wallet'
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
