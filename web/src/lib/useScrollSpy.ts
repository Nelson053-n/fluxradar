import { useEffect, useState } from 'react'

/**
 * Scroll-spy: возвращает id наиболее видимой секции из списка.
 * Использует IntersectionObserver; пересоздаётся при смене набора id
 * (секции появляются после загрузки данных).
 */
export function useScrollSpy(sectionIds: string[], enabled: boolean): string {
  const [active, setActive] = useState(sectionIds[0] ?? '')
  const key = sectionIds.join(',')

  useEffect(() => {
    if (!enabled) return
    const elements = sectionIds
      .map((id) => document.getElementById(id))
      .filter((el): el is HTMLElement => el != null)
    if (elements.length === 0) return

    // Держим карту видимости и выбираем верхнюю видимую секцию.
    const visible = new Map<string, number>()
    const observer = new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          if (e.isIntersecting) visible.set(e.target.id, e.intersectionRatio)
          else visible.delete(e.target.id)
        }
        // Первая по порядку из sectionIds, которая сейчас видна.
        const next = sectionIds.find((id) => visible.has(id))
        if (next) setActive(next)
      },
      { rootMargin: '-30% 0px -55% 0px', threshold: [0, 0.25, 0.5, 1] },
    )
    elements.forEach((el) => observer.observe(el))
    return () => observer.disconnect()
    // key отражает состав секций; enabled — готовность DOM.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key, enabled])

  return active
}
