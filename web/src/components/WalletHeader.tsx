import { useState } from 'react'
import { CopyIcon, RefreshIcon, SearchIcon, WalletIcon } from './icons'
import { useT } from '../i18n/store'

interface WalletHeaderProps {
  address: string
  /** Сабмит нового адреса из поля поиска. */
  onSubmit: (address: string) => void
  onRefresh: () => void
  refreshing: boolean
  /** Есть ли загруженные данные кошелька — для disabled-состояния Copy/Refresh/индикатора. */
  hasData: boolean
}

/** Wallet header: поиск адреса, пиктограмма кошелька, адрес, действия. Виден всегда. */
export function WalletHeader({ address, onSubmit, onRefresh, refreshing, hasData }: WalletHeaderProps) {
  const t = useT()
  const [copied, setCopied] = useState(false)
  const [value, setValue] = useState(address)
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
    if (trimmed) onSubmit(trimmed)
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
          className="flex min-w-0 flex-1 items-center gap-2 rounded-xl border border-border bg-subtle py-1.5 pl-3 pr-1.5 focus-within:border-border-strong md:flex-none"
        >
          <SearchIcon className="shrink-0 text-text-dim" />
          <input
            value={value}
            onChange={(e) => setValue(e.target.value)}
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
