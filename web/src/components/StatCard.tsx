import type { ReactNode } from 'react'
import { cardBase } from './cardStyles'

interface StatCardProps {
  label: string
  icon: ReactNode
  /** Основное значение (моно-шрифт). Уже отформатированная строка или ReactNode. */
  value: ReactNode
  /** Подпись под значением. */
  sub?: ReactNode
  className?: string
}

/** Базовая стат-карточка: иконка-лейбл → крупное моно-значение → подпись. */
export function StatCard({ label, icon, value, sub, className }: StatCardProps) {
  return (
    <div className={`${cardBase}${className ? ` ${className}` : ''}`}>
      <div className="mb-[18px] flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.08em] text-text-dim">
        <span className="flex h-8 w-8 items-center justify-center rounded-lg border border-[rgba(79,215,232,0.2)] bg-gradient-to-br from-[rgba(43,97,209,0.2)] to-[rgba(79,215,232,0.15)] text-flux-cyan">
          {icon}
        </span>
        {label}
      </div>
      <div className="mb-1.5 font-mono text-[32px] font-extrabold leading-none tracking-[-0.04em] md:text-[42px]">
        {value}
      </div>
      {sub != null && (
        <div className="flex items-center gap-1.5 text-[13px] text-text-secondary">
          {sub}
        </div>
      )}
    </div>
  )
}
