// 中文词典。键的完整性由编译器检查 (Record<Keys, string>)。
import type { Keys } from './store'

const zh: Record<Keys, string> = {
  // Nav
  'nav.dashboard': '仪表盘',
  'nav.allNodes': '所有节点',
  'nav.guide': '指南',
  'nav.demo': '演示',

  // Header / search
  'search.placeholder': 'Flux 地址 (t1… / t3…)',
  'search.aria': '钱包地址',
  'search.track': '追踪',
  'header.flux': 'FLUX',

  // Theme / lang
  'theme.toggleToLight': '切换到浅色主题',
  'theme.toggleToDark': '切换到深色主题',
  'lang.label': '语言',

  // Wallet header
  'wallet.label': '钱包',
  'wallet.autoRefresh': '自动刷新 · 60秒',
  'wallet.copy': '复制地址',
  'wallet.copied': '已复制！',
  'wallet.refresh': '刷新',

  // Stat cards
  'stat.totalNodes': '节点总数',
  'stat.totalNodes.sub': '在网络中活跃',
  'stat.walletBalance': '钱包余额',
  'stat.walletBalance.sub': 'FLUX · ≈ {usd} USD',
  'stat.fleetUptime': '节点运行时间',
  'stat.benchPassed': '基准测试通过',
  'stat.benchPassed.sub': '个节点通过基准测试',
  'stat.hostedApps': '托管应用',
  'stat.hostedApps.sub': '个应用部署在各节点',
  'stat.notAvailable': '可在节点详情中查看',
  'stat.counting': '正在统计应用…',

  // Tier card
  'tier.title': '等级分布',

  // Earnings
  'earnings.title': '预计收益',
  'earnings.daily': '每日',
  'earnings.monthly': '每月',
  'earnings.yearly': '每年',
  'earnings.period.day': '日',
  'earnings.period.month': '月',
  'earnings.period.year': '年',
  'earnings.apy': '抵押年化收益率 (APY)',
  'earnings.notAvailable': '暂不可用',
  'earnings.estimate': '估算',

  // Parallel Assets
  'pa.title': 'Parallel Assets',
  'pa.summary': '占总收益的 {pct}%，分布于 {count} 条链',
  'pa.perChainYear': '≈ 每条链每年 FLUX',
  'pa.share': '每条 ≈ {pct}% 占比',
  'pa.realSubtitle': '各链真实收益 (FLUX)',
  'pa.claimable': '可领取',
  'pa.claimed': '已领取',
  'pa.received': '已到账',
  'pa.fees': '手续费',

  // Totals
  'totals.mined': '累计挖矿',
  'totals.claimed': '累计已领取',
  'totals.claimable': '可领取',
  'totals.note': '基于节点运行时长估算',
  'totals.noteReal': '来自 Parallel Assets 数据',

  // Spotlight
  'spotlight.title': '焦点节点',
  'spotlight.subtitle': '你的杰出节点',
  'spotlight.oldest': '最早的节点',
  'spotlight.oldest.title': '运行最久的节点',
  'spotlight.highestRank': '最高排名',
  'spotlight.highestRank.title': '下一个待发放收益',
  'spotlight.mostHosted': '托管最多',
  'spotlight.mostHosted.title': '单节点应用数',
  'spotlight.payoutEta': '距发放',
  'spotlight.noData': '暂无数据',

  // Nodes table
  'nodes.title': '节点概览',
  'nodes.showing': '显示 {shown} / {total} · 点击行查看详情',
  'nodes.filter.all': '全部',
  'nodes.filter.cumulus': 'Cumulus',
  'nodes.filter.nimbus': 'Nimbus',
  'nodes.filter.stratus': 'Stratus',
  'nodes.searchPlaceholder': '按 IP 搜索…',
  'nodes.col.ip': 'IP',
  'nodes.col.tier': '等级',
  'nodes.col.rank': '排名',
  'nodes.col.status': '状态',
  'nodes.col.age': '运行时长',
  'nodes.col.lastPaid': '上次收益',
  'nodes.col.apps': '应用',
  'nodes.col.location': '位置',
  'nodes.col.fluxos': 'FluxOS',
  'nodes.col.maintenance': '维护',
  'nodes.col.payout': '距收益',
  'nodes.maintClosed': '已关闭',
  'nodes.payoutSoon': '即将',
  'nodes.payoutIn': '{dur}',
  'nodes.noMatch': '没有符合此筛选的节点。',
  'nodes.page': '第',
  'nodes.pageOf': '页 / 共',
  'nodes.prev': '上一页',
  'nodes.next': '下一页',
  'nodes.sortAsc': '升序',
  'nodes.sortDesc': '降序',
  'nodes.sortBy': '按{col}排序',

  // Tiers (display labels)
  'tierName.cumulus': 'Cumulus',
  'tierName.nimbus': 'Nimbus',
  'tierName.stratus': 'Stratus',

  // Price tooltip (FLUX year chart)
  'price.history.title': 'FLUX · 一年',
  'price.history.low': '最低',
  'price.history.high': '最高',
  'price.history.now': '当前',
  'price.history.loading': '正在加载价格历史…',
  'price.history.error': '价格历史不可用',

  // Telegram CTA
  'telegram.title': '在 Telegram 接收提醒',
  'telegram.body':
    '第一时间获知节点离线、基准测试失败或收益到账。无需在站点注册 — 直接通过机器人绑定。',
  'telegram.open': '打开 @FluxRadar_bot',

  // Footer
  'footer.copyright': '© 2026 FluxRadar · 自托管 Flux 节点监控',
  'footer.github': 'GitHub',
  'footer.apiDocs': 'API 文档',
  'footer.privacy': '隐私',
  'footer.fluxNetwork': 'Flux 网络',

  // 捐赠
  'donate.title': '支持 FluxRadar',
  'donate.text': 'FluxRadar 免费且自托管。如果它对你有帮助，欢迎用 FLUX 捐赠以支持持续开发。',
  'donate.address': 'FLUX 地址',
  'donate.copy': '复制地址',
  'donate.copied': '已复制！',

  // Privacy modal
  'privacy.title': '隐私',
  'privacy.body':
    'FluxRadar 从不索取也不存储私钥或助记词 — 仅使用公开的钱包地址。地址仅用于查询 Flux 网络的公开数据，不会与第三方共享。',
  'privacy.close': '关闭',

  // States
  'state.loadingDashboard': '正在加载仪表盘',
  'error.title': '加载失败',
  'error.retry': '重试',
  'error.unknown': '加载数据时发生未知错误',
  'empty.title': '未找到节点',
  'empty.body': '此钱包尚未关联任何 Flux 节点。',

  // Guide
  'guide.title': '使用方法',
  'guide.subtitle': '三步监控你的节点集群',
  'guide.step1.title': '输入钱包地址',
  'guide.step1.body':
    '在搜索栏粘贴公开的 Flux 钱包地址 (t1… 或 t3…)。无需注册，无需私钥 — 仅限公开地址。',
  'guide.step2.title': '查看你的节点',
  'guide.step2.body':
    '一目了然地查看整个集群：等级分布、收益队列排名、余额和每个节点的详情。',
  'guide.step3.title': '绑定 Telegram 提醒',
  'guide.step3.body':
    '打开 @FluxRadar_bot 并发送 /link <钱包>，即可在节点离线或收益到账时立即收到提醒。',

  // Node detail drawer
  'drawer.title': '节点详情',
  'drawer.close': '关闭',
  'drawer.status': '状态',
  'drawer.tier': '等级',
  'drawer.location': '位置',
  'drawer.apps': '托管的应用',
  'drawer.appsCount': '应用数量',
  'drawer.fluxos': 'FluxOS',
  'drawer.payout': '下次收益',
  'drawer.maintenance': '维护',
  'drawer.noApps': '没有托管应用',
  'drawer.noLocation': '位置不可用',
  'drawer.loading': '正在加载节点详情…',
  'drawer.error': '无法加载节点详情。',
  'drawer.retry': '重试',
}

export default zh
