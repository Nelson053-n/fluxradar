// English dictionary — source of truth for translation keys.
// `type Keys = keyof typeof en` derives the key set; ru/zh are checked against it.
const en = {
  // Nav
  'nav.dashboard': 'Dashboard',
  'nav.allNodes': 'All Nodes',
  'nav.guide': 'Guide',
  'nav.bot': 'Bot',

  // Header / search
  'search.placeholder': 'Flux address (t1… / t3…)',
  'search.aria': 'Wallet address',
  'search.track': 'Track',
  'header.flux': 'FLUX',

  // Theme / lang
  'theme.toggleToLight': 'Switch to light theme',
  'theme.toggleToDark': 'Switch to dark theme',
  'lang.label': 'Language',

  // Wallet header
  'wallet.label': 'Wallet',
  'wallet.autoRefresh': 'Auto-refresh · 60s',
  'wallet.copy': 'Copy address',
  'wallet.copied': 'Copied!',
  'wallet.refresh': 'Refresh',

  // Stat cards
  'stat.totalNodes': 'Total Nodes',
  'stat.totalNodes.sub': 'active in the network',
  'stat.walletBalance': 'Wallet Balance',
  'stat.walletBalance.sub': 'FLUX · ≈ {usd} USD',
  'stat.fleetUptime': 'Nodes Uptime',
  'stat.benchPassed': 'Bench Passed',
  'stat.benchPassed.sub': 'nodes passed benchmark',
  'stat.hostedApps': 'Hosted Apps',
  'stat.hostedApps.sub': 'apps across nodes',
  'stat.notAvailable': 'available in node details',
  'stat.counting': 'Counting apps…',

  // Tier card
  'tier.title': 'Tier Breakdown',

  // Earnings
  'earnings.title': 'Estimated Earnings',
  'earnings.daily': 'Daily',
  'earnings.monthly': 'Monthly',
  'earnings.yearly': 'Yearly',
  'earnings.period.day': 'Day',
  'earnings.period.month': 'Month',
  'earnings.period.year': 'Year',
  'earnings.apy': 'APY on collateral',
  'earnings.notAvailable': 'not yet available',
  'earnings.estimate': 'est.',

  // Parallel Assets
  'pa.title': 'Parallel Assets',
  'pa.summary': '{pct}% of total rewards across {count} chains',
  'pa.perChainYear': '≈ FLUX / year per chain',
  'pa.share': 'each ≈ {pct}% share',
  'pa.realSubtitle': 'Real per-chain rewards (FLUX)',
  'pa.claimable': 'Claimable',
  'pa.claimed': 'Claimed',
  'pa.received': 'Received',
  'pa.fees': 'Fees',

  // Totals
  'totals.mined': 'Total Mined',
  'totals.claimed': 'Total Claimed to date',
  'totals.claimable': 'Total Claimable',
  'totals.note': 'estimated from node age',
  'totals.noteReal': 'from Parallel Assets data',

  // Spotlight
  'spotlight.title': 'Spotlight',
  'spotlight.subtitle': 'Your standout nodes',
  'spotlight.oldest': 'Oldest Node',
  'spotlight.oldest.title': 'Longest-running node',
  'spotlight.highestRank': 'Highest Rank',
  'spotlight.highestRank.title': 'Next in payout queue',
  'spotlight.mostHosted': 'Most Hosted',
  'spotlight.mostHosted.title': 'Apps on a single node',
  'spotlight.payoutEta': 'Payout in',
  'spotlight.noData': 'no data available',

  // Nodes table
  'nodes.title': 'Node Overview',
  'nodes.showing': 'Showing {shown} of {total} · click row for details',
  'nodes.filter.all': 'All',
  'nodes.filter.cumulus': 'Cumulus',
  'nodes.filter.nimbus': 'Nimbus',
  'nodes.filter.stratus': 'Stratus',
  'nodes.searchPlaceholder': 'search by IP…',
  'nodes.col.ip': 'IP',
  'nodes.col.tier': 'Tier',
  'nodes.col.rank': 'Rank',
  'nodes.col.status': 'Status',
  'nodes.col.age': 'Age',
  'nodes.col.lastPaid': 'Last paid',
  'nodes.col.apps': 'Apps',
  'nodes.col.location': 'Location',
  'nodes.col.fluxos': 'FluxOS',
  'nodes.col.maintenance': 'Maintenance',
  'nodes.col.payout': 'Payout',
  'nodes.maintClosed': 'Closed',
  'nodes.payoutSoon': 'soon',
  'nodes.payoutIn': '{dur}',
  'nodes.noMatch': 'No nodes match this filter.',
  'nodes.page': 'Page',
  'nodes.pageOf': 'of',
  'nodes.prev': 'Prev',
  'nodes.next': 'Next',
  'nodes.sortAsc': 'sorted ascending',
  'nodes.sortDesc': 'sorted descending',
  'nodes.sortBy': 'sort by {col}',

  // Tiers (display labels)
  'tierName.cumulus': 'Cumulus',
  'tierName.nimbus': 'Nimbus',
  'tierName.stratus': 'Stratus',

  // Price tooltip (FLUX year chart)
  'price.history.title': 'FLUX · 1 year',
  'price.history.low': 'Low',
  'price.history.high': 'High',
  'price.history.now': 'Now',
  'price.history.loading': 'Loading price history…',
  'price.history.error': 'Price history unavailable',

  // Telegram CTA
  'telegram.title': 'Get alerts on Telegram',
  'telegram.body':
    'Be the first to know when a node drops offline, when the benchmark fails, or when a payout hits your wallet. No registration on the site — link directly via the bot.',
  'telegram.open': 'Open @FluxRadar_bot',

  // Footer
  'footer.copyright': '© 2026 FluxRadar · Self-hosted Flux node monitor',
  'footer.github': 'GitHub',
  'footer.apiDocs': 'API docs',
  'footer.privacy': 'Privacy',
  'footer.fluxNetwork': 'Flux Network',

  // Донаты
  'donate.title': 'Support FluxRadar',
  'donate.text': 'FluxRadar is free and self-hosted. If it helps you, consider a donation in FLUX — it keeps development going.',
  'donate.address': 'FLUX address',
  'donate.copy': 'Copy address',
  'donate.copied': 'Copied!',
  'donate.visitorsTotal': 'Total visitors',
  'donate.visitorsToday': 'Visitors today',

  // Privacy modal
  'privacy.title': 'Privacy',
  'privacy.body':
    'FluxRadar never asks for and never stores private keys or seed phrases — only public wallet addresses. Addresses are used solely to query public Flux network data and are not shared with third parties.',
  'privacy.close': 'Close',

  // States
  'state.loadingDashboard': 'Loading dashboard',
  'error.title': 'Failed to load',
  'error.retry': 'Retry',
  'error.unknown': 'Unknown error while loading data',
  'empty.title': 'No nodes found',
  'empty.body': 'This wallet has no Flux nodes associated with it yet.',

  // Guide
  'guide.title': 'How it works',
  'guide.subtitle': 'Three steps to monitor your fleet',
  'guide.step1.title': 'Enter a wallet address',
  'guide.step1.body':
    'Paste any public Flux wallet address (t1… or t3…) into the search bar. No registration, no private keys — public addresses only.',
  'guide.step2.title': 'Watch your nodes',
  'guide.step2.body':
    'See your whole fleet at a glance: tier breakdown, rank in the payout queue, balance and per-node details.',
  'guide.step3.title': 'Link Telegram alerts',
  'guide.step3.body':
    'Open @FluxRadar_bot and send /link <wallet> to get instant alerts when a node drops offline or a payout arrives.',

  // Node detail drawer
  'drawer.title': 'Node details',
  'drawer.close': 'Close',
  'drawer.status': 'Status',
  'drawer.tier': 'Tier',
  'drawer.location': 'Location',
  'drawer.apps': 'Hosted apps',
  'drawer.appsCount': 'Apps count',
  'drawer.fluxos': 'FluxOS',
  'drawer.payout': 'Next payout',
  'drawer.maintenance': 'Maintenance',
  'drawer.noApps': 'No apps hosted',
  'drawer.noLocation': 'Location unavailable',
  'drawer.loading': 'Loading node details…',
  'drawer.error': 'Could not load node details.',
  'drawer.retry': 'Retry',
} as const

export default en
