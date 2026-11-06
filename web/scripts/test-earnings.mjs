import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import ts from 'typescript'

const source = readFileSync(new URL('../src/lib/earningsCalc.ts', import.meta.url), 'utf8')
const js = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.ESNext } }).outputText
const { calcEarnings } = await import(`data:text/javascript;base64,${Buffer.from(js).toString('base64')}`)
const fleet = { cumulus: 1, nimbus: 8, stratus: 0 }
const network = { cumulus: 3389, nimbus: 1699, stratus: 1823 }
const r = calcEarnings(fleet, network, 0.060868)
assert.equal(r.available, true)
assert.ok(Math.abs(r.monthlyUsd - 150.85878284174096) < 1e-8)
assert.ok(Math.abs(r.monthlyFlux - (0.9 / 3389 + 8 * 3.15 / 1699) * 86400 * 1.9) < 1e-8)
assert.ok(Math.abs(r.monthlyFlux - r.dailyFlux * 30) < 1e-8)
assert.equal(calcEarnings(fleet, null, 0.06).available, false)
assert.equal(calcEarnings(fleet, network, null).available, false)
assert.equal(calcEarnings(fleet, { ...network, nimbus: 0 }, 0.06).available, false)
assert.equal(calcEarnings({ cumulus: 0, nimbus: 0, stratus: 0 }, network, 0.06).monthlyFlux, 0)
console.log('earnings: current rewards, 9 PA, 30 days and unavailable inputs OK')
