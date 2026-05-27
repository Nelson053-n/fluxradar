// Типизированный клиент нашего REST API (через Vite dev-proxy /api → :5049).
// Прямых запросов к Flux API из браузера нет (CORS/кэш/бизнес-логика, §3.3 ТЗ).

export type Tier = 'CUMULUS' | 'NIMBUS' | 'STRATUS'

export interface PriceResponse {
  flux_usd: number
  /** Изменение цены за 24ч, % (может быть отрицательным). */
  change_24h: number
}

/** Счётчик уникальных посетителей сайта: всего и за сутки. */
export interface VisitorStats {
  total: number
  today: number
}

/** Число активных нод сети по тирам (для калькулятора). */
export interface NetworkNodes {
  total: number
  cumulus: number
  nimbus: number
  stratus: number
}

/** Одна точка истории цены: unix-секунды + цена в USD. */
export interface PricePoint {
  t: number
  usd: number
}

/** История цены FLUX за год (~366 точек) для спарклайна в Header. */
export interface PriceHistoryResponse {
  points: PricePoint[]
}

/** Денежная величина: количество FLUX + USD-эквивалент. */
export interface Money {
  flux: number
  usd: number
}

/** Доход Parallel Assets по одному чейну (значения в FLUX). */
export interface PaChain {
  /** Тикер чейна в нижнем регистре: kda, eth, bsc, trx, sol, avax, erg, algo, matic, base. */
  chain: string
  claimable: number
  claimed: number
  received: number
  /** Комиссия по чейну (в FLUX) — различается между чейнами. */
  fees: number
}

export interface WalletSummary {
  total_nodes: number
  tiers: {
    cumulus: number
    nimbus: number
    stratus: number
  }
  balance: Money
  /** Оценка дохода (est., прогноз по механике наград — не факт выплат). */
  earnings: {
    daily: Money
    monthly: Money
    yearly: Money
    /** Годовой APY, % от стоимости залога (collateral). */
    apy_percent: number
    /** Доля боковой доходности (Parallel Assets), %. */
    parallel_assets_pct: number
    /** Число PA-чейнов. */
    parallel_assets_count: number
  }
  /** Накопленные суммы за жизнь нод. */
  totals: {
    /** Всего намайнено за всё время. */
    mined: Money
    /** Получено к настоящему моменту. */
    claimed: Money
    /** Накоплено к получению с последней выплаты. */
    claimable: Money
    /** true — значения оценочные (est.); false — реальные данные Parallel Assets fusion. */
    is_estimate: boolean
  }
  /** Реальная разбивка Parallel Assets по чейнам (10 чейнов). Может быть пуст. */
  pa_chains: PaChain[]
  /** Сводка сети для контекста. */
  network: {
    total: number
    cumulus: number
    nimbus: number
    stratus: number
  }
  /** Возраст старейшей ноды флота, сек. */
  oldest_node_age_secs: number
  /** Доля нод флота «в норме», %. */
  fleet_uptime_pct: number
  /** Сколько нод флота прошли бенчмарк. */
  bench_passed: number
  /** Всего приложений, размещённых на нодах флота. */
  hosted_apps: number
  /** Изменение цены FLUX за 24ч, %. */
  price_change_24h: number
}

export interface FluxNode {
  ip: string
  tier: Tier
  rank: number
  payment_address: string
  /** Возраст ноды (с момента активации), сек. */
  age_secs: number
  /** Unix-timestamp последней выплаты (или null). */
  last_paid: number | null
  /** Статус ноды (нода в детерминированном списке = CONFIRMED). */
  status: string
  /** Время до следующей выплаты, сек. 0 = вот-вот. */
  payout_eta_secs: number
  /** Окно обслуживания (сек до переподтверждения); null = закрыто/нет данных. */
  maintenance_window_secs: number | null
  /** Версия FluxOS (или null). */
  flux_os_version: string | null
  /** Геолокация ноды (или null). */
  geo: { country: string; country_code: string; city: string } | null
}

export interface NodesResponse {
  nodes: FluxNode[]
}

/** Деталь ноды (ленивые дорогие данные: статус, apps, гео). */
export interface NodeDetail {
  ip: string
  status: string | null
  tier: string | null
  /** Число приложений на ноде (согласовано с длиной apps). */
  apps_count: number
  apps: string[]
  geo: {
    country: string
    country_code: string
    city: string
  } | null
}

interface ApiError {
  error: string
}

/** Ошибка с сообщением от бэкенда (напр. «невалидный адрес кошелька»). */
export class ApiRequestError extends Error {
  readonly status: number
  constructor(message: string, status: number) {
    super(message)
    this.name = 'ApiRequestError'
    this.status = status
  }
}

async function getJson<T>(path: string, signal?: AbortSignal): Promise<T> {
  let res: Response
  try {
    res = await fetch(path, { signal, headers: { Accept: 'application/json' } })
  } catch {
    throw new ApiRequestError('API недоступен — проверьте подключение', 0)
  }

  if (!res.ok) {
    let message = `Ошибка запроса (${res.status})`
    try {
      const body = (await res.json()) as ApiError
      if (body?.error) message = body.error
    } catch {
      // тело без JSON — оставляем дефолтное сообщение
    }
    throw new ApiRequestError(message, res.status)
  }

  return (await res.json()) as T
}

export function fetchPrice(signal?: AbortSignal): Promise<PriceResponse> {
  return getJson<PriceResponse>('/api/v1/network/price', signal)
}

/** История цены FLUX за год — тянется лениво при первом ховере на цену. */
export function fetchPriceHistory(signal?: AbortSignal): Promise<PriceHistoryResponse> {
  return getJson<PriceHistoryResponse>('/api/v1/network/price/history', signal)
}

/** Учесть текущего посетителя и получить счётчики (всего / за сутки). */
export function fetchVisitorStats(signal?: AbortSignal): Promise<VisitorStats> {
  return getJson<VisitorStats>('/api/v1/stats/visitors', signal)
}

/** Число активных нод сети по тирам — для калькулятора без загруженного кошелька. */
export function fetchNetworkNodes(signal?: AbortSignal): Promise<NetworkNodes> {
  return getJson<NetworkNodes>('/api/v1/network/nodes', signal)
}

export function fetchWalletSummary(
  address: string,
  signal?: AbortSignal,
): Promise<WalletSummary> {
  return getJson<WalletSummary>(
    `/api/v1/wallet/${encodeURIComponent(address)}/summary`,
    signal,
  )
}

export function fetchWalletNodes(
  address: string,
  signal?: AbortSignal,
): Promise<NodesResponse> {
  return getJson<NodesResponse>(
    `/api/v1/wallet/${encodeURIComponent(address)}/nodes`,
    signal,
  )
}

/** Деталь конкретной ноды (дорогие данные, тянутся лениво по клику). */
export function fetchNodeDetail(
  ip: string,
  signal?: AbortSignal,
): Promise<NodeDetail> {
  return getJson<NodeDetail>(
    `/api/v1/node/${encodeURIComponent(ip)}/detail`,
    signal,
  )
}

/** Сводка приложений флота: всего на всех нодах + максимум на одной ноде. */
export interface WalletAppsResponse {
  /** Всего приложений на нодах флота. */
  total: number
  /** Максимум приложений на одной ноде. */
  max_on_node: number
  /** Нода с максимумом приложений (IP + tier + count); null если приложений нет. */
  top_node: { ip: string; tier: string; count: number } | null
}

/**
 * Число приложений флота. Запрос МЕДЛЕННЫЙ (секунды–десятки секунд) —
 * грузится отдельно от основного дашборда, не блокируя его.
 */
export function fetchWalletApps(
  address: string,
  signal?: AbortSignal,
): Promise<WalletAppsResponse> {
  return getJson<WalletAppsResponse>(
    `/api/v1/wallet/${encodeURIComponent(address)}/apps`,
    signal,
  )
}
