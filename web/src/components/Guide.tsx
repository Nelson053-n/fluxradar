import { SearchIcon, MonitorIcon, SendIcon } from './icons'
import { cardBase } from './cardStyles'
import { useT, type Keys } from '../i18n/store'

const STEPS: { icon: typeof SearchIcon; titleKey: Keys; bodyKey: Keys }[] = [
  { icon: SearchIcon, titleKey: 'guide.step1.title', bodyKey: 'guide.step1.body' },
  { icon: MonitorIcon, titleKey: 'guide.step2.title', bodyKey: 'guide.step2.body' },
  { icon: SendIcon, titleKey: 'guide.step3.title', bodyKey: 'guide.step3.body' },
]

/** Guide — короткая инструкция «как пользоваться» в стиле остальных карточек. */
export function Guide() {
  const t = useT()
  return (
    <section className="mt-12">
      <div className="mb-5 flex items-end justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-[-0.02em]">{t('guide.title')}</h2>
          <div className="mt-1 text-[13px] text-text-dim">{t('guide.subtitle')}</div>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-5 md:grid-cols-3">
        {STEPS.map((step, i) => {
          const Icon = step.icon
          return (
            <div key={step.titleKey} className={cardBase}>
              <div className="mb-4 flex items-center gap-3">
                <span className="flex h-9 w-9 items-center justify-center rounded-lg border border-[rgba(79,215,232,0.2)] bg-gradient-to-br from-[rgba(43,97,209,0.2)] to-[rgba(79,215,232,0.15)] text-flux-cyan">
                  <Icon />
                </span>
                <span className="font-mono text-sm font-semibold text-text-dim">
                  {String(i + 1).padStart(2, '0')}
                </span>
              </div>
              <h3 className="mb-2 text-base font-bold text-text-primary">{t(step.titleKey)}</h3>
              <p className="text-sm leading-relaxed text-text-secondary">{t(step.bodyKey)}</p>
            </div>
          )
        })}
      </div>
    </section>
  )
}
