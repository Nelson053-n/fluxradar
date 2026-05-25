/**
 * Логотип-радар FluxRadar: круглый радар с вращающейся развёрткой.
 * Вся визуальная начинка (sweep + кольца прицела) — в .logo-mark (index.css),
 * портирована 1-в-1 из нового мокапа. Анимация radarSweep отключается при
 * prefers-reduced-motion (см. index.css).
 */
export function RadarLogo() {
  return <div className="logo-mark" aria-hidden="true" />
}
