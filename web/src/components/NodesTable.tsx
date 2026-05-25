import { useMemo, useState } from 'react'
import type { FluxNode, Tier } from '../api/client'
import { SearchIcon, ArrowRightIcon } from './icons'
import {
  NA,
  formatInt,
  formatAge,
  formatDuration,
  countryFlag,
  statusTone,
} from '../lib/format'
import { statusPill, statusPillTone, statusDotTone } from './cardStyles'
import { useI18n, type Keys } from '../i18n/store'

interface NodesTableProps {
  nodes: FluxNode[]
  /** Клик по строке — открыть drawer с деталями ноды. */
  onSelect: (node: FluxNode) => void
}

const PAGE_SIZE = 50

type FilterKey = 'all' | 'cumulus' | 'nimbus' | 'stratus'

/** Колонки, по которым доступна сортировка. */
type SortKey =
  | 'ip'
  | 'tier'
  | 'rank'
  | 'status'
  | 'age'
  | 'payout'
  | 'location'
  | 'fluxos'
  | 'maintenance'
type SortDir = 'asc' | 'desc'

const tierCellClass: Record<Tier, string> = {
  CUMULUS: 'text-flux-cyan before:bg-flux-cyan before:shadow-[0_0_6px_var(--flux-cyan)]',
  NIMBUS: 'text-flux-glow before:bg-flux-glow before:shadow-[0_0_6px_var(--flux-glow)]',
  STRATUS: 'text-flux-purple before:bg-flux-purple before:shadow-[0_0_6px_var(--flux-purple)]',
}

const tierNameKey: Record<Tier, Keys> = {
  CUMULUS: 'tierName.cumulus',
  NIMBUS: 'tierName.nimbus',
  STRATUS: 'tierName.stratus',
}

// Порядок тиров для сортировки (Cumulus < Nimbus < Stratus).
const tierOrder: Record<Tier, number> = { CUMULUS: 0, NIMBUS: 1, STRATUS: 2 }

// Колонки таблицы (бэкенд теперь отдаёт apps/location/fluxos/payout/maintenance).
const COLUMNS: { labelKey: Keys; sort: SortKey | null }[] = [
  { labelKey: 'nodes.col.ip', sort: 'ip' },
  { labelKey: 'nodes.col.tier', sort: 'tier' },
  { labelKey: 'nodes.col.rank', sort: 'rank' },
  { labelKey: 'nodes.col.status', sort: 'status' },
  { labelKey: 'nodes.col.age', sort: 'age' },
  { labelKey: 'nodes.col.payout', sort: 'payout' },
  { labelKey: 'nodes.col.location', sort: 'location' },
  { labelKey: 'nodes.col.fluxos', sort: 'fluxos' },
  { labelKey: 'nodes.col.maintenance', sort: 'maintenance' },
]

// Строка локации для сортировки/поиска: страна (или город), либо '' если нет гео.
function locationKey(n: FluxNode): string {
  if (!n.geo) return ''
  return n.geo.country || n.geo.city || ''
}

// Сравнение двух нод по выбранному ключу (всегда asc; направление применяется снаружи).
// null-значения (нет данных) сортируются в конец при asc через -Infinity / ''.
function compareNodes(a: FluxNode, b: FluxNode, key: SortKey): number {
  switch (key) {
    case 'ip':
      return a.ip.localeCompare(b.ip, undefined, { numeric: true })
    case 'tier':
      return tierOrder[a.tier] - tierOrder[b.tier]
    case 'rank':
      return a.rank - b.rank
    case 'status':
      return a.status.localeCompare(b.status)
    case 'age':
      return a.age_secs - b.age_secs
    case 'payout':
      return a.payout_eta_secs - b.payout_eta_secs
    case 'location':
      return locationKey(a).localeCompare(locationKey(b))
    case 'fluxos':
      return (a.flux_os_version ?? '').localeCompare(b.flux_os_version ?? '')
    case 'maintenance':
      return (a.maintenance_window_secs ?? -Infinity) - (b.maintenance_window_secs ?? -Infinity)
  }
}

/** Node Overview: фильтр по тиру + поиск по IP + сортировка по колонкам + пагинация (по 50 строк). */
export function NodesTable({ nodes, onSelect }: NodesTableProps) {
  const { t, lang } = useI18n()
  const [filter, setFilter] = useState<FilterKey>('all')
  const [query, setQuery] = useState('')
  const [page, setPage] = useState(0)
  const [sortKey, setSortKey] = useState<SortKey>('rank')
  const [sortDir, setSortDir] = useState<SortDir>('asc')

  const counts = useMemo(() => {
    const c = { all: nodes.length, cumulus: 0, nimbus: 0, stratus: 0 }
    for (const n of nodes) {
      if (n.tier === 'CUMULUS') c.cumulus++
      else if (n.tier === 'NIMBUS') c.nimbus++
      else if (n.tier === 'STRATUS') c.stratus++
    }
    return c
  }, [nodes])

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase()
    return nodes.filter((n) => {
      if (filter !== 'all' && n.tier.toLowerCase() !== filter) return false
      if (q && !n.ip.toLowerCase().includes(q)) return false
      return true
    })
  }, [nodes, filter, query])

  const sorted = useMemo(() => {
    const dir = sortDir === 'asc' ? 1 : -1
    // copy перед сортировкой — не мутируем filtered.
    return [...filtered].sort((a, b) => compareNodes(a, b, sortKey) * dir)
  }, [filtered, sortKey, sortDir])

  const pageCount = Math.max(1, Math.ceil(sorted.length / PAGE_SIZE))
  const safePage = Math.min(page, pageCount - 1)
  const pageRows = sorted.slice(safePage * PAGE_SIZE, safePage * PAGE_SIZE + PAGE_SIZE)

  function selectFilter(key: FilterKey) {
    setFilter(key)
    setPage(0)
  }

  // Клик по заголовку: та же колонка — переключить направление; другая — выбрать её asc.
  function toggleSort(key: SortKey) {
    if (key === sortKey) {
      setSortDir((d) => (d === 'asc' ? 'desc' : 'asc'))
    } else {
      setSortKey(key)
      setSortDir('asc')
    }
    setPage(0)
  }

  const tabs: { key: FilterKey; label: string; count: number }[] = [
    { key: 'all', label: t('nodes.filter.all'), count: counts.all },
    { key: 'cumulus', label: t('nodes.filter.cumulus'), count: counts.cumulus },
    { key: 'nimbus', label: t('nodes.filter.nimbus'), count: counts.nimbus },
    { key: 'stratus', label: t('nodes.filter.stratus'), count: counts.stratus },
  ]

  return (
    <>
      <div className="mb-5 flex items-end justify-between">
        <div>
          <h2 className="text-xl font-bold tracking-[-0.02em] md:text-2xl">{t('nodes.title')}</h2>
          <div className="mt-1 text-[13px] text-text-dim">
            {t('nodes.showing', {
              shown: formatInt(pageRows.length),
              total: formatInt(sorted.length),
            })}
          </div>
        </div>
      </div>

      <div className="overflow-hidden rounded-2xl border border-border bg-bg-card backdrop-blur-xl">
        {/* toolbar — на мобильном в столбик: вкладки (горизонтальный скролл) + поиск на всю ширину */}
        <div className="flex flex-col gap-3 border-b border-border px-4 py-4 sm:flex-row sm:flex-wrap sm:items-center sm:justify-between sm:px-5">
          <div className="thin-scroll -mx-1 flex gap-1 overflow-x-auto rounded-[10px] bg-overlay p-1 sm:mx-0">
            {tabs.map((tab) => (
              <button
                key={tab.key}
                onClick={() => selectFilter(tab.key)}
                className={`whitespace-nowrap rounded-[7px] px-3.5 py-1.5 text-[13px] font-medium transition-colors ${
                  filter === tab.key
                    ? 'bg-subtle-hover text-text-primary'
                    : 'text-text-secondary hover:text-text-primary'
                }`}
              >
                {tab.label}
                <span className="ml-1.5 font-mono text-[11px] opacity-60">{tab.count}</span>
              </button>
            ))}
          </div>
          <div className="flex w-full items-center gap-2 rounded-lg border border-border bg-overlay px-3 py-2 sm:w-60">
            <SearchIcon className="shrink-0 text-text-dim" />
            <input
              value={query}
              onChange={(e) => {
                setQuery(e.target.value)
                setPage(0)
              }}
              spellCheck={false}
              placeholder={t('nodes.searchPlaceholder')}
              className="w-full bg-transparent font-mono text-[13px] text-text-primary outline-none placeholder:text-text-dim"
            />
          </div>
        </div>

        {/* mobile (<md): список карточек вместо таблицы — нет горизонтального скролла */}
        <div className="flex flex-col gap-3 p-4 md:hidden">
          {pageRows.map((n) => (
            <NodeCard key={`${n.ip}-${n.rank}`} node={n} onSelect={onSelect} />
          ))}
          {pageRows.length === 0 && (
            <div className="px-2 py-10 text-center text-sm text-text-dim">{t('nodes.noMatch')}</div>
          )}
        </div>

        {/* desktop (md+): таблица */}
        <div className="thin-scroll hidden overflow-x-auto md:block">
          <table className="w-full table-fixed border-collapse">
            <colgroup>
              <col className="w-[15%]" />
              <col className="w-[11%]" />
              <col className="w-[8%]" />
              <col className="w-[9%]" />
              <col className="w-[9%]" />
              <col className="w-[14%]" />
              <col className="w-[11%]" />
              <col className="w-[11%]" />
              <col className="w-[12%]" />
            </colgroup>
            <thead className="bg-overlay-soft">
              <tr>
                {COLUMNS.map((col) => {
                  const isActive = col.sort != null && col.sort === sortKey
                  const indicator = isActive ? (sortDir === 'asc' ? '▲' : '▼') : ''
                  const base =
                    'border-b border-border px-3 py-2.5 text-left text-[11px] font-semibold uppercase tracking-[0.08em] text-text-dim'
                  if (col.sort == null) {
                    return (
                      <th key={col.labelKey} className={base}>
                        {t(col.labelKey)}
                      </th>
                    )
                  }
                  const sortable = col.sort
                  return (
                    <th key={col.labelKey} className={`${base} p-0`}>
                      <button
                        type="button"
                        onClick={() => toggleSort(sortable)}
                        aria-label={t('nodes.sortBy', { col: t(col.labelKey) })}
                        aria-sort={
                          isActive ? (sortDir === 'asc' ? 'ascending' : 'descending') : 'none'
                        }
                        className={`flex w-full items-center gap-1 px-3 py-2.5 text-left text-[11px] font-semibold uppercase tracking-[0.08em] transition-colors hover:text-text-secondary ${
                          isActive ? 'text-text-primary' : 'text-text-dim'
                        }`}
                      >
                        <span className="truncate">{t(col.labelKey)}</span>
                        <span className="font-mono text-[9px] leading-none text-flux-glow">
                          {indicator}
                        </span>
                      </button>
                    </th>
                  )
                })}
              </tr>
            </thead>
            <tbody>
              {pageRows.map((n) => {
                const tone = statusTone(n.status)
                return (
                  <tr
                    key={`${n.ip}-${n.rank}`}
                    onClick={() => onSelect(n)}
                    className="cursor-pointer border-b border-border transition-colors last:border-b-0 hover:bg-row-hover"
                  >
                    <td
                      className="truncate px-3 py-3 font-mono text-xs text-text-primary"
                      title={n.ip}
                    >
                      {n.ip}
                    </td>
                    <td className="px-3 py-3">
                      <span
                        className={`inline-flex items-center gap-1 text-[13px] font-semibold tracking-[0.02em] before:h-1.5 before:w-1.5 before:shrink-0 before:rounded-full before:content-[''] ${tierCellClass[n.tier]}`}
                      >
                        {t(tierNameKey[n.tier])}
                      </span>
                    </td>
                    <td className="px-3 py-3 text-left font-mono text-[13px] text-text-primary">
                      #{n.rank}
                    </td>
                    <td className="px-3 py-3">
                      <span className={`${statusPill} ${statusPillTone[tone]} gap-1 px-2 py-0.5`}>
                        <span className={`h-1.5 w-1.5 shrink-0 rounded-full ${statusDotTone[tone]}`} />
                        {n.status}
                      </span>
                    </td>
                    <td className="px-3 py-3 font-mono text-[13px] text-text-secondary">
                      {formatAge(n.age_secs, lang)}
                    </td>
                    <td className="px-3 py-3 font-mono text-[13px] text-text-secondary">
                      {n.payout_eta_secs <= 0
                        ? t('nodes.payoutSoon')
                        : t('nodes.payoutIn', { dur: formatDuration(n.payout_eta_secs, lang) })}
                    </td>
                    <td className="px-3 py-3 font-mono text-[13px] text-text-secondary">
                      {n.geo ? (
                        <span className="flex items-center gap-1">
                          <span className="shrink-0 leading-none">
                            {countryFlag(n.geo.country_code)}
                          </span>
                          <span className="truncate">{n.geo.country || n.geo.city}</span>
                        </span>
                      ) : (
                        <span className="text-text-dim">{NA}</span>
                      )}
                    </td>
                    <td
                      className="truncate px-3 py-3 font-mono text-[13px] text-text-secondary"
                      title={n.flux_os_version ?? undefined}
                    >
                      {n.flux_os_version ?? <span className="text-text-dim">{NA}</span>}
                    </td>
                    <td className="px-3 py-3 font-mono text-[13px] text-text-secondary">
                      {n.maintenance_window_secs != null ? (
                        formatDuration(n.maintenance_window_secs, lang)
                      ) : (
                        <span className="text-text-dim">{t('nodes.maintClosed')}</span>
                      )}
                    </td>
                  </tr>
                )
              })}
              {pageRows.length === 0 && (
                <tr>
                  <td
                    colSpan={COLUMNS.length}
                    className="px-5 py-12 text-center text-sm text-text-dim"
                  >
                    {t('nodes.noMatch')}
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>

        {/* footer / pagination */}
        <div className="flex flex-wrap items-center justify-between gap-3 border-t border-border bg-overlay-soft px-4 py-4 sm:px-5">
          <div className="text-[13px] text-text-dim">
            {t('nodes.page')} <span className="font-mono text-text-secondary">{safePage + 1}</span>{' '}
            {t('nodes.pageOf')} <span className="font-mono text-text-secondary">{pageCount}</span>
          </div>
          <div className="flex items-center gap-2">
            <button
              type="button"
              disabled={safePage === 0}
              onClick={() => setPage((p) => Math.max(0, p - 1))}
              className="inline-flex items-center gap-1.5 rounded-lg border border-[rgba(91,141,239,0.3)] bg-[rgba(43,97,209,0.12)] px-4 py-2 text-[13px] font-semibold text-flux-glow transition-opacity disabled:cursor-not-allowed disabled:opacity-40"
            >
              {t('nodes.prev')}
            </button>
            <button
              type="button"
              disabled={safePage >= pageCount - 1}
              onClick={() => setPage((p) => Math.min(pageCount - 1, p + 1))}
              className="inline-flex items-center gap-1.5 rounded-lg border border-[rgba(91,141,239,0.3)] bg-[rgba(43,97,209,0.12)] px-4 py-2 text-[13px] font-semibold text-flux-glow transition-opacity disabled:cursor-not-allowed disabled:opacity-40"
            >
              {t('nodes.next')}
              <ArrowRightIcon />
            </button>
          </div>
        </div>
      </div>
    </>
  )
}

/**
 * Мобильная карточка ноды (<md) — замена строки таблицы.
 * Сверху: IP (моно, крупнее) + статус-бейдж справа. Ниже — пары label:value
 * в 2 колонки. Клик открывает тот же drawer, что и строка таблицы (onSelect).
 */
function NodeCard({ node: n, onSelect }: { node: FluxNode; onSelect: (node: FluxNode) => void }) {
  const { t, lang } = useI18n()
  const tone = statusTone(n.status)
  return (
    <button
      type="button"
      onClick={() => onSelect(n)}
      className="w-full rounded-xl border border-border bg-bg-card p-4 text-left backdrop-blur-xl transition-colors hover:border-border-strong hover:bg-row-hover"
    >
      <div className="mb-3 flex items-center justify-between gap-3">
        <span className="min-w-0 truncate font-mono text-[15px] font-semibold text-text-primary" title={n.ip}>
          {n.ip}
        </span>
        <span className={`${statusPill} ${statusPillTone[tone]} shrink-0 gap-1 px-2 py-0.5`}>
          <span className={`h-1.5 w-1.5 shrink-0 rounded-full ${statusDotTone[tone]}`} />
          {n.status}
        </span>
      </div>

      <dl className="grid grid-cols-2 gap-x-4 gap-y-2.5 text-[13px]">
        <CardField label={t('nodes.col.tier')}>
          <span
            className={`inline-flex items-center gap-1 font-semibold tracking-[0.02em] before:h-1.5 before:w-1.5 before:shrink-0 before:rounded-full before:content-[''] ${tierCellClass[n.tier]}`}
          >
            {t(tierNameKey[n.tier])}
          </span>
        </CardField>

        <CardField label={t('nodes.col.rank')}>
          <span className="font-mono text-text-primary">#{n.rank}</span>
        </CardField>

        <CardField label={t('nodes.col.payout')}>
          <span className="font-mono text-text-secondary">
            {n.payout_eta_secs <= 0
              ? t('nodes.payoutSoon')
              : t('nodes.payoutIn', { dur: formatDuration(n.payout_eta_secs, lang) })}
          </span>
        </CardField>

        <CardField label={t('nodes.col.age')}>
          <span className="font-mono text-text-secondary">{formatAge(n.age_secs, lang)}</span>
        </CardField>

        <CardField label={t('nodes.col.location')}>
          {n.geo ? (
            <span className="flex min-w-0 items-center gap-1 font-mono text-text-secondary">
              <span className="shrink-0 leading-none">{countryFlag(n.geo.country_code)}</span>
              <span className="truncate">{n.geo.country || n.geo.city}</span>
            </span>
          ) : (
            <span className="text-text-dim">{NA}</span>
          )}
        </CardField>

        <CardField label={t('nodes.col.fluxos')}>
          <span className="truncate font-mono text-text-secondary" title={n.flux_os_version ?? undefined}>
            {n.flux_os_version ?? <span className="text-text-dim">{NA}</span>}
          </span>
        </CardField>

        <CardField label={t('nodes.col.maintenance')}>
          <span className="font-mono text-text-secondary">
            {n.maintenance_window_secs != null ? (
              formatDuration(n.maintenance_window_secs, lang)
            ) : (
              <span className="text-text-dim">{t('nodes.maintClosed')}</span>
            )}
          </span>
        </CardField>
      </dl>
    </button>
  )
}

/** Пара label/value в мобильной карточке ноды. */
function CardField({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex min-w-0 flex-col gap-0.5">
      <dt className="text-[10px] font-semibold uppercase tracking-[0.08em] text-text-dim">{label}</dt>
      <dd className="min-w-0 truncate">{children}</dd>
    </div>
  )
}
