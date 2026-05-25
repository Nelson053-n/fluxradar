import { SendIcon } from './icons'
import { useT } from '../i18n/store'

/** Telegram CTA — привязка через бота, без регистрации на сайте (§3.3 ТЗ). */
export function TelegramCta() {
  const t = useT()
  return (
    <div className="relative mt-8 flex flex-wrap items-center justify-between gap-6 overflow-hidden rounded-[20px] border border-[rgba(79,215,232,0.2)] bg-gradient-to-br from-[rgba(43,97,209,0.1)] to-[rgba(79,215,232,0.05)] p-5 after:pointer-events-none after:absolute after:-right-[10%] after:-top-[40%] after:h-[400px] after:w-[400px] after:rounded-full after:bg-[radial-gradient(circle,rgba(79,215,232,0.15),transparent_70%)] after:content-[''] sm:gap-8 sm:p-8">
      <div className="relative z-[1] min-w-0 flex-1">
        <h3 className="mb-2 text-xl font-bold tracking-[-0.02em] md:text-2xl">{t('telegram.title')}</h3>
        <p className="max-w-[560px] leading-relaxed text-text-secondary">{t('telegram.body')}</p>
      </div>
      <a
        href="https://t.me/FluxRadar_bot"
        target="_blank"
        rel="noopener noreferrer"
        className="relative z-[1] flex w-full items-center justify-center gap-3 rounded-xl border border-border-strong bg-white/[0.04] px-[22px] py-3.5 font-mono text-sm text-text-primary transition-all hover:-translate-y-px hover:bg-white/[0.08] sm:w-auto"
      >
        <SendIcon className="text-flux-cyan" />
        {t('telegram.open')}
      </a>
    </div>
  )
}
