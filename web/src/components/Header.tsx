import { useRef, useState } from 'react'
import { ThemeToggle } from './ThemeToggle'
import { LangSwitch } from './LangSwitch'
import { RadarLogo } from './RadarLogo'
import { PriceSparkline } from './PriceSparkline'
import { formatPrice, formatPctChange } from '../lib/format'
import { fetchPriceHistory, type PricePoint } from '../api/client'
import { useT, type Keys } from '../i18n/store'

interface HeaderProps {
  priceUsd: number | null
  /** Изменение цены FLUX за 24ч, % (null — ещё не загружено). */
  priceChange24h: number | null
  /** Скролл к секции дашборда по id (Dashboard/All Nodes/Guide). */
  onNavigate: (sectionId: string) => void
  /** Пролистать к Telegram-секции и открыть бота в новом окне (кнопка «Бот»). */
  onBot: () => void
  /** id активной (видимой) секции — для подсветки вкладки. */
  activeSection: string
}

type NavItem =
  | { kind: 'scroll'; labelKey: Keys; section: string }
  | { kind: 'bot'; labelKey: Keys }

const NAV: NavItem[] = [
  { kind: 'scroll', labelKey: 'nav.dashboard', section: 'overview' },
  { kind: 'scroll', labelKey: 'nav.allNodes', section: 'nodes' },
  { kind: 'scroll', labelKey: 'nav.guide', section: 'guide' },
  { kind: 'bot', labelKey: 'nav.bot' },
]

const activeTab =
  'whitespace-nowrap rounded-full bg-gradient-to-br from-flux-primary to-flux-glow px-3 py-2 text-[13px] font-medium text-white shadow-[0_4px_16px_rgba(43,97,209,0.4)] lg:px-[18px] lg:text-sm'
const idleTab =
  'whitespace-nowrap rounded-full px-3 py-2 text-[13px] font-medium text-text-secondary transition-colors hover:bg-white/5 hover:text-text-primary lg:px-[18px] lg:text-sm'

/** Header: лого-радар + nav + переключатели темы/языка + цена FLUX. */
export function Header({ priceUsd, priceChange24h, onNavigate, onBot, activeSection }: HeaderProps) {
  const t = useT()

  // История цены за год — грузится лениво при первом ховере/фокусе на блоке цены,
  // потом кэшируется в state (повторно не запрашиваем).
  const [showPriceTip, setShowPriceTip] = useState(false)
  const [history, setHistory] = useState<PricePoint[] | null>(null)
  const [historyError, setHistoryError] = useState(false)
  const historyRequested = useRef(false)

  function loadHistory() {
    if (historyRequested.current) return
    historyRequested.current = true
    fetchPriceHistory()
      .then((res) => setHistory(res.points))
      .catch(() => setHistoryError(true))
  }

  function openPriceTip() {
    setShowPriceTip(true)
    loadHistory()
  }

  return (
    <header className="flex flex-wrap items-center justify-between gap-x-4 gap-y-3 py-4 pb-10">
      <div className="flex min-w-0 flex-wrap items-center gap-x-4 gap-y-3">
        <div className="flex min-w-0 shrink-0 items-center gap-3 text-xl font-extrabold tracking-tight">
          <RadarLogo />
          <span className="whitespace-nowrap bg-gradient-to-r from-flux-primary to-flux-cyan bg-clip-text text-transparent">
            FluxRadar
          </span>
        </div>

        <nav className="hidden shrink-0 flex-nowrap items-center gap-0.5 rounded-full border border-border bg-[var(--bg-card)] p-1.5 backdrop-blur-xl md:flex lg:gap-1">
          {NAV.map((item) => {
            const isActive = item.kind === 'scroll' && item.section === activeSection
            return (
              <button
                key={item.labelKey}
                type="button"
                onClick={() => (item.kind === 'bot' ? onBot() : onNavigate(item.section))}
                className={isActive ? activeTab : idleTab}
              >
                {t(item.labelKey)}
              </button>
            )
          })}
        </nav>
      </div>

      <div className="flex min-w-0 flex-wrap items-center justify-end gap-x-3 gap-y-2">
        <LangSwitch />
        <ThemeToggle />

        {/* Компактная цена FLUX без тултипа — только до lg (на узких экранах тултип не помещается). */}
        <div className="flex items-center gap-2 whitespace-nowrap rounded-xl border border-border bg-[var(--bg-card)] px-3 py-2 text-[13px] backdrop-blur-xl lg:hidden">
          <span className="font-medium text-text-dim">{t('header.flux')}</span>
          <span className="font-mono font-semibold text-text-primary">
            {priceUsd != null ? formatPrice(priceUsd) : '—'}
          </span>
          {priceChange24h != null && <PriceChangeBadge value={priceChange24h} />}
        </div>

        {/* Блок цены FLUX + ленивый тултип со спарклайном за год (Task 4) — от lg. */}
        <div
          className="relative hidden lg:block"
          onMouseEnter={openPriceTip}
          onMouseLeave={() => setShowPriceTip(false)}
        >
          <button
            type="button"
            onFocus={openPriceTip}
            onBlur={() => setShowPriceTip(false)}
            aria-label={t('price.history.title')}
            className="flex items-center gap-2.5 whitespace-nowrap rounded-xl border border-border bg-[var(--bg-card)] px-4 py-2 text-sm backdrop-blur-xl transition-colors hover:border-border-strong focus:outline-none focus-visible:border-border-strong"
          >
            <span className="font-medium text-text-dim">{t('header.flux')}</span>
            <span className="font-mono font-semibold text-text-primary">
              {priceUsd != null ? formatPrice(priceUsd) : '—'}
            </span>
            {priceChange24h != null && <PriceChangeBadge value={priceChange24h} />}
          </button>

          {showPriceTip && (
            <div className="absolute right-0 top-[calc(100%+8px)] z-20">
              <PriceSparkline points={history} error={historyError} />
            </div>
          )}
        </div>
      </div>
    </header>
  )
}

/** Бейдж изменения цены за 24ч: зелёный плюс / красный минус / нейтральный ноль. */
function PriceChangeBadge({ value }: { value: number }) {
  const { text, tone } = formatPctChange(value)
  const cls =
    tone === 'up' ? 'text-success' : tone === 'down' ? 'text-danger' : 'text-text-dim'
  return <span className={`font-mono text-xs font-semibold ${cls}`}>{text}</span>
}
