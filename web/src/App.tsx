import { useCallback, useEffect, useRef, useState } from 'react'
import {
  ApiRequestError,
  fetchPrice,
  fetchWalletApps,
  fetchWalletNodes,
  fetchWalletSummary,
  type FluxNode,
  type PriceResponse,
  type WalletAppsResponse,
  type WalletSummary,
} from './api/client'
import { Header } from './components/Header'
import { WalletHeader } from './components/WalletHeader'
import { StatCard } from './components/StatCard'
import { TierCard } from './components/TierCard'
import { EarningsCard } from './components/EarningsCard'
import { ParallelAssets } from './components/ParallelAssets'
import { TotalsCards } from './components/TotalsCards'
import { Spotlight } from './components/Spotlight'
import { NodesTable } from './components/NodesTable'
import { NodeDetailDrawer } from './components/NodeDetailDrawer'
import { RadarSpinner } from './components/RadarSpinner'
import { TelegramCta } from './components/TelegramCta'
import { Guide } from './components/Guide'
import { Footer } from './components/Footer'
import { DashboardSkeleton, EmptyFleet, ErrorState } from './components/Skeleton'
import {
  MonitorIcon,
  WalletIcon,
  ClockIcon,
  CheckSquareIcon,
  ShieldIcon,
} from './components/icons'
import { formatInt, formatNum, formatUsd } from './lib/format'
import { readWallet, writeWallet, walletFromUrl } from './lib/walletParam'
import { useScrollSpy } from './lib/useScrollSpy'
import { useT } from './i18n/store'

// Дефолтный кошелёк при первой загрузке — с небольшим флотом, чтобы страница
// открывалась быстрее (меньше нод → быстрее первый ответ API).
const DEFAULT_ADDRESS = 't1MRWH9Q9sWA1XvqmwGoWqynaFFquq8pb7K'

// Секции для якорной навигации и scroll-spy.
const SECTIONS = ['overview', 'nodes', 'guide']

interface WalletData {
  summary: WalletSummary
  nodes: FluxNode[]
}

function App() {
  const t = useT()
  // Lazy init: ?wallet= → localStorage → DEFAULT_ADDRESS (фикс потери адреса на F5).
  const [address, setAddress] = useState(() => readWallet(DEFAULT_ADDRESS))
  const [price, setPrice] = useState<PriceResponse | null>(null)
  const [data, setData] = useState<WalletData | null>(null)
  const [loading, setLoading] = useState(true)
  const [refreshing, setRefreshing] = useState(false)
  const [error, setError] = useState<string | null>(null)
  // Нода, для которой открыт drawer с деталями (null — закрыт).
  const [selectedNode, setSelectedNode] = useState<FluxNode | null>(null)
  // Число приложений флота — медленный запрос, грузится отдельно после дашборда.
  // status: pending (запрос в полёте / ещё не стартовал) | ready | error.
  const [apps, setApps] = useState<{
    status: 'pending' | 'ready' | 'error'
    data: WalletAppsResponse | null
  }>({ status: 'pending', data: null })
  // Сброс apps-состояния при смене кошелька — «adjust state during render»
  // вместо синхронного setState в эффекте (react-hooks/set-state-in-effect).
  const [appsAddress, setAppsAddress] = useState(address)
  if (address !== appsAddress) {
    setAppsAddress(address)
    setApps({ status: 'pending', data: null })
  }
  const priceUsd = price?.flux_usd ?? null

  // Есть ли непустой флот — чтобы не считать приложения для пустого кошелька.
  const hasFleet = data != null && data.summary.total_nodes > 0 && data.nodes.length > 0

  // Отменяем устаревшие запросы при быстрой смене адреса.
  const abortRef = useRef<AbortController | null>(null)

  // load() только грузит данные и пишет результат — спиннер выставляют
  // вызывающие (обработчики событий). На mount начальный стейт уже loading:true,
  // поэтому эффект не делает синхронный setState (react-hooks/set-state-in-effect).
  const load = useCallback(async (addr: string) => {
    abortRef.current?.abort()
    const ctrl = new AbortController()
    abortRef.current = ctrl

    try {
      const [summary, nodesRes] = await Promise.all([
        fetchWalletSummary(addr, ctrl.signal),
        fetchWalletNodes(addr, ctrl.signal),
      ])
      if (ctrl.signal.aborted) return
      setData({ summary, nodes: nodesRes.nodes })
    } catch (e) {
      if (ctrl.signal.aborted) return
      const msg =
        e instanceof ApiRequestError ? e.message : 'Неизвестная ошибка при загрузке данных'
      setError(msg)
      setData(null)
    } finally {
      if (!ctrl.signal.aborted) {
        setLoading(false)
        setRefreshing(false)
      }
    }
  }, [])

  // Цена FLUX (+ изменение за 24ч) — независимо от выбранного кошелька.
  useEffect(() => {
    const ctrl = new AbortController()
    fetchPrice(ctrl.signal)
      .then((p) => setPrice(p))
      .catch(() => {
        /* цена некритична для дашборда */
      })
    return () => ctrl.abort()
  }, [])

  // Первичная загрузка стартового кошелька (из URL/localStorage/дефолт).
  // Делаем fetch инлайн внутри эффекта, чтобы единственные setState шли после await.
  useEffect(() => {
    const ctrl = new AbortController()
    abortRef.current = ctrl
    const initial = readWallet(DEFAULT_ADDRESS)
    void (async () => {
      try {
        const [summary, nodesRes] = await Promise.all([
          fetchWalletSummary(initial, ctrl.signal),
          fetchWalletNodes(initial, ctrl.signal),
        ])
        if (ctrl.signal.aborted) return
        setData({ summary, nodes: nodesRes.nodes })
      } catch (e) {
        if (ctrl.signal.aborted) return
        setError(
          e instanceof ApiRequestError ? e.message : 'Неизвестная ошибка при загрузке данных',
        )
      } finally {
        if (!ctrl.signal.aborted) setLoading(false)
      }
    })()
    return () => ctrl.abort()
  }, [])

  // Кнопка «назад/вперёд» браузера → синхронизируем адрес с ?wallet= и перезагружаем.
  useEffect(() => {
    function onPopState() {
      const fromUrl = walletFromUrl()
      if (fromUrl && fromUrl !== address) {
        setAddress(fromUrl)
        setLoading(true)
        setError(null)
        void load(fromUrl)
      }
    }
    window.addEventListener('popstate', onPopState)
    return () => window.removeEventListener('popstate', onPopState)
  }, [address, load])

  // Дозагрузка числа приложений флота — отдельно от дашборда (запрос медленный).
  // Стартует, когда для текущего адреса уже есть непустой флот. Сброс при смене
  // кошелька делается в render-фазе выше; здесь setState идёт только после await
  // (react-hooks/set-state-in-effect требует не звать setState синхронно в теле).
  useEffect(() => {
    if (!hasFleet) return
    const ctrl = new AbortController()
    void (async () => {
      try {
        const res = await fetchWalletApps(address, ctrl.signal)
        if (ctrl.signal.aborted) return
        setApps({ status: 'ready', data: res })
      } catch {
        if (ctrl.signal.aborted) return
        setApps({ status: 'error', data: null })
      }
    })()
    return () => ctrl.abort()
  }, [address, hasFleet])

  const handleSubmit = useCallback(
    (addr: string) => {
      setAddress(addr)
      writeWallet(addr)
      setLoading(true)
      setError(null)
      void load(addr)
    },
    [load],
  )

  const handleRefresh = useCallback(() => {
    setRefreshing(true)
    setError(null)
    void load(address)
  }, [address, load])

  const handleRetry = useCallback(() => {
    setLoading(true)
    setError(null)
    void load(address)
  }, [address, load])

  // Кнопка «Бот» — пролистать вниз к Telegram-секции.
  const handleBot = useCallback(() => {
    document.getElementById('telegram')?.scrollIntoView({ behavior: 'smooth', block: 'start' })
  }, [])

  // Якорная навигация Header → плавный скролл к секции (отступ сверху задаёт scroll-mt у секции).
  const handleNavigate = useCallback((sectionId: string) => {
    document.getElementById(sectionId)?.scrollIntoView({ behavior: 'smooth', block: 'start' })
  }, [])

  // Scroll-spy активен только когда секции реально в DOM (данные загружены).
  const hasContent = !loading && !error && data != null
  const activeSection = useScrollSpy(SECTIONS, hasContent)

  // Денежные значения с учётом текущей цены, если бэкенд не прислал usd.
  const usdValue = (flux: number, fallbackUsd: number) =>
    priceUsd != null ? flux * priceUsd : fallbackUsd

  // Суммарно доступно к получению (claimable) по всем параллельным чейнам, FLUX.
  const paClaimable = (data?.summary.pa_chains ?? []).reduce((s, c) => s + c.claimable, 0)

  // Значение для плашек, зависящих от медленного подсчёта приложений:
  // спиннер пока считаем (запрос в полёте), число когда готово, «—» при ошибке.
  const appsValue = (pick: (a: WalletAppsResponse) => number) => {
    if (apps.status === 'ready' && apps.data) return formatInt(pick(apps.data))
    if (apps.status === 'error') return '—'
    // pending при наличии флота — запрос ещё считается.
    if (hasFleet) return <RadarSpinner label={t('stat.counting')} />
    return '—'
  }

  return (
    <div className="relative z-[1] mx-auto max-w-[1440px] px-4 pb-16 pt-6 sm:px-6 md:px-12">
      <Header
        priceUsd={priceUsd}
        priceChange24h={price?.change_24h ?? null}
        onNavigate={handleNavigate}
        onBot={handleBot}
        activeSection={activeSection}
      />

      <WalletHeader
        address={address}
        onSubmit={handleSubmit}
        onRefresh={handleRefresh}
        refreshing={refreshing}
        hasData={hasContent}
      />

      {loading && <DashboardSkeleton />}

      {!loading && error && <ErrorState message={error} onRetry={handleRetry} />}

      {!loading && !error && data && (
        <>
          {data.summary.total_nodes === 0 || data.nodes.length === 0 ? (
            <EmptyFleet />
          ) : (
            <>
              {/* Ряд 1: Total Nodes · Tier Breakdown · Earnings(span 2) */}
              <div
                id="overview"
                className="stagger stagger-grid mb-8 grid scroll-mt-24 grid-cols-1 gap-5 md:grid-cols-2 xl:grid-cols-4"
              >
                <StatCard
                  label={t('stat.totalNodes')}
                  icon={<MonitorIcon />}
                  value={formatInt(data.summary.total_nodes)}
                  sub={<span className="text-text-dim">{t('stat.totalNodes.sub')}</span>}
                />
                <TierCard
                  cumulus={data.summary.tiers.cumulus}
                  nimbus={data.summary.tiers.nimbus}
                  stratus={data.summary.tiers.stratus}
                />
                <EarningsCard earnings={data.summary.earnings} />
              </div>

              {/* Ряд 2: Wallet Balance(real) · Fleet Uptime · Bench · Hosted Apps */}
              <div className="stagger stagger-grid mb-8 grid grid-cols-1 gap-5 md:grid-cols-2 xl:grid-cols-4">
                <StatCard
                  label={t('stat.walletBalance')}
                  icon={<WalletIcon />}
                  value={formatNum(data.summary.balance.flux, 2)}
                  sub={
                    <span className="flex flex-col gap-0.5">
                      <span className="text-text-secondary">
                        {t('stat.walletBalance.sub', {
                          usd: formatUsd(usdValue(data.summary.balance.flux, data.summary.balance.usd)),
                        })}
                      </span>
                      {paClaimable > 0 && (
                        <span className="text-[11px] text-text-dim">
                          {t('stat.walletBalance.pa', { flux: formatNum(paClaimable, 2) })}
                        </span>
                      )}
                    </span>
                  }
                />
                <StatCard
                  label={t('stat.fleetUptime')}
                  icon={<ClockIcon />}
                  value={`${formatNum(data.summary.fleet_uptime_pct, 1)}%`}
                  sub={<span className="text-text-dim">{t('stat.totalNodes.sub')}</span>}
                />
                <StatCard
                  label={t('stat.benchPassed')}
                  icon={<CheckSquareIcon />}
                  value={`${formatInt(data.summary.bench_passed)} / ${formatInt(data.summary.total_nodes)}`}
                  sub={<span className="text-text-dim">{t('stat.benchPassed.sub')}</span>}
                />
                <StatCard
                  label={t('stat.hostedApps')}
                  icon={<ShieldIcon />}
                  value={appsValue((a) => a.total)}
                  sub={<span className="text-text-dim">{t('stat.hostedApps.sub')}</span>}
                />
              </div>

              <Spotlight
                nodes={data.nodes}
                oldestAgeSecs={data.summary.oldest_node_age_secs}
                mostHostedValue={appsValue((a) => a.max_on_node)}
                mostHostedNode={apps.status === 'ready' ? (apps.data?.top_node ?? null) : null}
              />

              <div id="nodes" className="mb-8 scroll-mt-4">
                <NodesTable nodes={data.nodes} onSelect={setSelectedNode} />
              </div>

              {/* Ряд: накопленные оценки — Total Mined · Claimed · Claimable */}
              <div className="stagger stagger-grid mb-8 grid grid-cols-1 gap-5 md:grid-cols-3">
                <TotalsCards totals={data.summary.totals} />
              </div>
              {/* Parallel Assets — боковая доходность по PA-чейнам */}
              <ParallelAssets
                earnings={data.summary.earnings}
                paChains={data.summary.pa_chains ?? []}
              />
              <div id="telegram" className="scroll-mt-24">
                <TelegramCta />
              </div>
              <div id="guide" className="scroll-mt-24">
                <Guide />
              </div>
            </>
          )}
        </>
      )}

      <Footer />

      <NodeDetailDrawer node={selectedNode} onClose={() => setSelectedNode(null)} />
    </div>
  )
}

export default App
