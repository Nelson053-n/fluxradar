import { useState } from 'react'
import { CopyIcon, RefreshIcon, SearchIcon, WalletIcon } from './icons'
import { shortAddress } from '../lib/format'
import { useT } from '../i18n/store'

interface WalletHeaderProps {
  address: string
  /** Сабмит нового адреса из поля поиска. */
  onSubmit: (address: string) => void
  onRefresh: () => void
  refreshing: boolean
  /** Есть ли загруженные данные кошелька — для disabled-состояния Copy/Refresh/индикатора. */
  hasData: boolean
  /** История прошлых кошельков (для автодополнения). */
  history: string[]
  /** Удалить адрес из истории. */
  onRemoveHistory: (address: string) => void
}

/** Wallet header: поиск адреса, пиктограмма кошелька, адрес, действия. Виден всегда. */
export function WalletHeader({
  address,
  onSubmit,
  onRefresh,
  refreshing,
  hasData,
  history,
  onRemoveHistory,
}: WalletHeaderProps) {
  const t = useT()
  const [copied, setCopied] = useState(false)
  const [value, setValue] = useState(address)
  // Открыт ли дропдаун истории (по фокусу поля).
  const [open, setOpen] = useState(false)
  // Синхронизация поля ввода при внешней смене адреса (демо, ?wallet=, кнопка назад)
  // через паттерн «adjust state during render» вместо эффекта (см. react.dev).
  const [prevAddress, setPrevAddress] = useState(address)
  if (address !== prevAddress) {
    setPrevAddress(address)
    setValue(address)
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    const trimmed = value.trim()
    if (trimmed) {
      onSubmit(trimmed)
      setOpen(false)
    }
  }

  // Фильтр истории по текущему вводу; текущий загруженный адрес не показываем.
  const q = value.trim().toLowerCase()
  const suggestions = history.filter(
    (a) => a !== address && (q === '' || a.toLowerCase().includes(q)),
  )

  function pickSuggestion(addr: string) {
    setValue(addr)
    onSubmit(addr)
    setOpen(false)
  }

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(address)
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    } catch {
      // clipboard может быть недоступен — тихо игнорируем
    }
  }

  const iconBtn =
    'flex h-10 w-10 items-center justify-center rounded-[10px] border border-border bg-subtle text-text-secondary transition-colors enabled:hover:border-border-strong enabled:hover:bg-subtle-hover enabled:hover:text-text-primary disabled:cursor-not-allowed disabled:opacity-40'

  return (
    <div className="mb-8 flex flex-col gap-4 rounded-2xl border border-border bg-panel px-4 py-4 backdrop-blur-xl sm:px-6 sm:py-5 md:flex-row md:flex-wrap md:items-center md:justify-between">
      <div className="flex min-w-0 flex-1 items-center gap-3 sm:gap-4">
        <span className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl border border-[rgba(79,215,232,0.2)] bg-gradient-to-br from-[rgba(43,97,209,0.2)] to-[rgba(79,215,232,0.15)] text-flux-cyan">
          <WalletIcon width={22} height={22} />
        </span>

        <form
          onSubmit={handleSubmit}
          onFocus={() => setOpen(true)}
          // Закрываем, когда фокус ушёл из формы целиком (relatedTarget вне неё),
          // — клик по пункту/крестику внутри формы не закрывает дропдаун.
          onBlur={(e) => {
            if (!e.currentTarget.contains(e.relatedTarget as Node)) setOpen(false)
          }}
          className="relative flex min-w-0 flex-1 items-center gap-2 rounded-xl border border-border bg-subtle py-1.5 pl-3 pr-1.5 focus-within:border-border-strong md:flex-none"
        >
          <SearchIcon className="shrink-0 text-text-dim" />
          <input
            value={value}
            onChange={(e) => setValue(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Escape') setOpen(false)
            }}
            spellCheck={false}
            autoComplete="off"
            placeholder={t('search.placeholder')}
            aria-label={t('search.aria')}
            className="w-full min-w-0 bg-transparent font-mono text-sm text-text-primary outline-none placeholder:text-text-dim md:w-[320px] lg:w-[420px]"
          />
          <button
            type="submit"
            className="shrink-0 whitespace-nowrap rounded-lg bg-gradient-to-br from-flux-primary to-flux-glow px-4 py-2 text-sm font-semibold text-white shadow-[0_4px_20px_rgba(43,97,209,0.5)] transition-transform hover:-translate-y-px"
          >
            {t('search.track')}
          </button>

          {open && suggestions.length > 0 && (
            <ul
              role="listbox"
              className="absolute left-0 right-0 top-[calc(100%+6px)] z-30 max-h-72 overflow-auto rounded-xl border border-border bg-[var(--bg-elevated)] py-1 shadow-glow"
            >
              {suggestions.map((addr) => (
                <li key={addr} role="option" aria-selected={false} className="group flex items-center">
                  <button
                    type="button"
                    onClick={() => pickSuggestion(addr)}
                    className="min-w-0 flex-1 truncate px-3 py-2 text-left font-mono text-[13px] text-text-secondary transition-colors hover:bg-subtle-hover hover:text-text-primary"
                    title={addr}
                  >
                    {shortAddress(addr)}
                  </button>
                  <button
                    type="button"
                    aria-label={t('search.removeHistory')}
                    title={t('search.removeHistory')}
                    onClick={() => onRemoveHistory(addr)}
                    className="mr-1 flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-text-dim opacity-0 transition-opacity hover:bg-subtle-hover hover:text-danger group-hover:opacity-100"
                  >
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <line x1="18" y1="6" x2="6" y2="18" />
                      <line x1="6" y1="6" x2="18" y2="18" />
                    </svg>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </form>

        {hasData && (
          <div className="hidden min-w-0 lg:block">
            <div className="text-[11px] font-semibold uppercase tracking-[0.12em] text-text-dim">
              {t('wallet.label')}
            </div>
          </div>
        )}
      </div>

      <div className="flex items-center gap-2">
        <div
          className={`flex flex-1 items-center justify-center gap-2.5 rounded-[10px] border border-border bg-subtle px-3 py-2.5 text-[13px] text-text-secondary transition-opacity sm:flex-none sm:px-4 ${
            hasData ? '' : 'opacity-40'
          }`}
        >
          <span className="h-2 w-2 rounded-full bg-success shadow-[0_0_12px_var(--success)]" />
          {t('wallet.autoRefresh')}
        </div>
        <button
          type="button"
          onClick={handleCopy}
          disabled={!hasData}
          title={copied ? t('wallet.copied') : t('wallet.copy')}
          className={iconBtn}
        >
          <CopyIcon />
        </button>
        <button
          type="button"
          onClick={onRefresh}
          disabled={!hasData}
          title={t('wallet.refresh')}
          className={iconBtn}
        >
          <RefreshIcon className={refreshing ? 'animate-spin' : undefined} />
        </button>
      </div>
    </div>
  )
}
