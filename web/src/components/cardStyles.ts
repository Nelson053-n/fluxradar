// Общий класс стат-карточки (glassmorphism + верхняя световая линия .card-toptrim).
export const cardBase =
  'card-toptrim relative overflow-hidden rounded-2xl border border-border bg-bg-card p-6 backdrop-blur-xl transition-all duration-300 hover:-translate-y-0.5 hover:border-border-strong'

import type { StatusTone } from '../lib/format'

// Статус-бейдж ноды (пилюля + точка) — перенесён из .status-pill мокапа.
export const statusPill =
  'inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 font-mono text-[11px] font-semibold uppercase tracking-[0.06em]'

export const statusPillTone: Record<StatusTone, string> = {
  online: 'border-[rgba(47,211,160,0.25)] bg-[rgba(47,211,160,0.10)] text-success',
  offline: 'border-[rgba(255,107,107,0.25)] bg-[rgba(255,107,107,0.10)] text-danger',
  warning: 'border-[rgba(245,184,71,0.25)] bg-[rgba(245,184,71,0.10)] text-warning',
}

export const statusDotTone: Record<StatusTone, string> = {
  online: 'bg-success shadow-[0_0_8px_var(--success)]',
  offline: 'bg-danger',
  warning: 'bg-warning',
}
