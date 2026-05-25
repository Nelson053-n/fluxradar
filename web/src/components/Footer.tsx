import { useEffect, useState } from 'react'
import { useT, type Keys } from '../i18n/store'

// Внешние ссылки футера (открываются в новой вкладке). Privacy — модалка, не ссылка.
const EXTERNAL_LINKS: { labelKey: Keys; href: string }[] = [
  { labelKey: 'footer.github', href: 'https://github.com/RunOnFlux' },
  { labelKey: 'footer.apiDocs', href: 'https://docs.runonflux.io/' },
  { labelKey: 'footer.fluxNetwork', href: 'https://home.runonflux.io/' },
]

/** Flux-адрес для донатов. */
const DONATE_ADDRESS = 't1TfYPwJdZvKu6yHccqfJxLAsCKoZ9nfevr'

export function Footer() {
  const t = useT()
  const [privacyOpen, setPrivacyOpen] = useState(false)

  return (
    <>
      <DonateCard />
      <footer className="mt-16 flex flex-wrap items-center justify-between gap-4 border-t border-border pt-8 text-[13px] text-text-dim">
        <div>{t('footer.copyright')}</div>
        <div className="flex flex-wrap gap-6">
          {EXTERNAL_LINKS.map(({ labelKey, href }) => (
            <a
              key={labelKey}
              href={href}
              target="_blank"
              rel="noopener noreferrer"
              className="text-text-dim transition-colors hover:text-text-primary"
            >
              {t(labelKey)}
            </a>
          ))}
          <button
            type="button"
            onClick={() => setPrivacyOpen(true)}
            className="text-text-dim transition-colors hover:text-text-primary"
          >
            {t('footer.privacy')}
          </button>
        </div>
      </footer>

      {privacyOpen && <PrivacyModal onClose={() => setPrivacyOpen(false)} />}
    </>
  )
}

function PrivacyModal({ onClose }: { onClose: () => void }) {
  const t = useT()

  // Esc закрывает модалку.
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') onClose()
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [onClose])

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-6">
      <div
        className="absolute inset-0 animate-fadeIn bg-black/50 backdrop-blur-sm"
        onClick={onClose}
        aria-hidden="true"
      />
      <div
        role="dialog"
        aria-modal="true"
        aria-label={t('privacy.title')}
        className="relative w-full max-w-md animate-fadeIn rounded-2xl border border-border bg-[var(--bg-card)] p-6 shadow-glow backdrop-blur-2xl"
      >
        <div className="mb-3 flex items-center justify-between">
          <div className="text-lg font-bold text-text-primary">{t('privacy.title')}</div>
          <button
            type="button"
            onClick={onClose}
            aria-label={t('privacy.close')}
            className="flex h-9 w-9 items-center justify-center rounded-lg border border-border text-text-dim transition-colors hover:border-border-strong hover:text-text-primary"
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
        <p className="text-sm leading-relaxed text-text-secondary">{t('privacy.body')}</p>
      </div>
    </div>
  )
}

/** Блок донатов: призыв + Flux-адрес с кнопкой копирования. */
function DonateCard() {
  const t = useT()
  const [copied, setCopied] = useState(false)

  async function copy() {
    try {
      await navigator.clipboard.writeText(DONATE_ADDRESS)
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    } catch {
      // clipboard недоступен — тихо игнорируем
    }
  }

  return (
    <div className="card-toptrim mt-16 overflow-hidden rounded-2xl border border-[rgba(79,215,232,0.25)] bg-gradient-to-br from-[rgba(43,97,209,0.12)] to-[rgba(79,215,232,0.06)] p-6 backdrop-blur-xl">
      <div className="mb-2 flex items-center gap-2 text-base font-bold text-text-primary">
        <span className="text-flux-cyan">♥</span>
        {t('donate.title')}
      </div>
      <p className="mb-4 max-w-2xl text-sm leading-relaxed text-text-secondary">
        {t('donate.text')}
      </p>
      <div className="flex flex-wrap items-center gap-3">
        <span className="text-[11px] font-semibold uppercase tracking-[0.08em] text-text-dim">
          {t('donate.address')}
        </span>
        <code className="min-w-0 break-all rounded-lg border border-border bg-subtle px-3 py-2 font-mono text-[13px] text-text-primary">
          {DONATE_ADDRESS}
        </code>
        <button
          type="button"
          onClick={copy}
          className="shrink-0 rounded-lg border border-border bg-subtle px-3 py-2 text-[13px] text-text-secondary transition-colors hover:border-border-strong hover:bg-subtle-hover hover:text-text-primary"
        >
          {copied ? t('donate.copied') : t('donate.copy')}
        </button>
      </div>
    </div>
  )
}
