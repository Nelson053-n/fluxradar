/**
 * Радар-спиннер в стиле логотипа FluxRadar: вращающаяся развёртка
 * (conic-gradient + radarSweep keyframes из index.css, класс .radar-spinner).
 * Единый индикатор загрузки данных во всех местах дашборда.
 */
interface RadarSpinnerProps {
  /** Подпись для скринридеров (loading-состояние). */
  label: string
  /** Размер: sm (~24px), md (базовый ~34px), lg (~64px). */
  size?: 'sm' | 'md' | 'lg'
  /** Доп. классы (напр. для отступов вокруг). */
  className?: string
}

const sizeClass: Record<NonNullable<RadarSpinnerProps['size']>, string> = {
  sm: 'radar-spinner--sm',
  md: '',
  lg: 'radar-spinner--lg',
}

export function RadarSpinner({ label, size = 'md', className }: RadarSpinnerProps) {
  const classes = ['radar-spinner', sizeClass[size], className].filter(Boolean).join(' ')
  return <span className={classes} role="status" aria-label={label} />
}
