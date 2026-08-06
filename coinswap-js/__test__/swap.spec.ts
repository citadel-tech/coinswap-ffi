import { execFileSync } from 'node:child_process'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'

import test from 'ava'

import { AddressType, type BackendConfig, type RpcConfig, Taker } from '../index'

// FFI taker integration test: 4 takers x 2 makers.
//
// Mirrors the Rust/Python/Swift swap tests: four takers run sequentially
// against the Docker regtest stack (1 RPC maker + 1 Electrum maker), covering
// the full backend x protocol matrix -- legacy/taproot over rpc/electrum. Each
// taker funds a fresh wallet and runs a 2-maker coinswap.
//
// Live-only: gated behind COINSWAP_LIVE_TESTS=1 (needs the Docker stack + a
// built native addon), otherwise skipped.

const liveTestsEnabled = process.env.COINSWAP_LIVE_TESTS === '1'

// Sats swapped per taker; funded with 4x this (1.0 BTC across 4 addresses).
const SWAP_AMOUNT = 500_000

const RPC_AUTH_ARGS = ['-regtest', '-rpcport=18442', '-rpcuser=user', '-rpcpassword=password']

function bitcoinCli(args: string[]): string {
  return execFileSync('docker', ['exec', 'coinswap-bitcoind', 'bitcoin-cli', ...args], {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  }).trim()
}

function cleanupWallet(walletName: string) {
  const walletsDir = path.join(os.homedir(), '.coinswap', 'taker', 'wallets')
  if (fs.existsSync(walletsDir)) {
    for (const entry of fs.readdirSync(walletsDir)) {
      if (!entry.startsWith(walletName)) continue
      fs.rmSync(path.join(walletsDir, entry), { recursive: true, force: true })
    }
  }
  try {
    bitcoinCli([...RPC_AUTH_ARGS, 'unloadwallet', walletName])
  } catch {
    // Ignore missing wallet errors.
  }
}

function fund(address: string) {
  bitcoinCli([...RPC_AUTH_ARGS, '-rpcwallet=test', 'sendtoaddress', address, '0.25'])
}

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

// Sync until spendable reaches `target`, tolerating Electrum indexing lag.
async function waitForSpendable(taker: Taker, target: number) {
  let balances = taker.getBalances()
  for (let i = 0; i < 30; i += 1) {
    taker.syncAndSave()
    balances = taker.getBalances()
    if (balances.spendable >= target) return balances
    await sleep(3_000)
  }
  return balances
}

type Backend = 'rpc' | 'electrum'

type SwapCase = {
  name: string
  backend: Backend
  protocol: 'Legacy' | 'Taproot'
  addressType: AddressType
}

const CASES: SwapCase[] = [
  { name: 'legacy_rpc', backend: 'rpc', protocol: 'Legacy', addressType: AddressType.P2WPKH },
  { name: 'taproot_rpc', backend: 'rpc', protocol: 'Taproot', addressType: AddressType.P2TR },
  { name: 'legacy_electrum', backend: 'electrum', protocol: 'Legacy', addressType: AddressType.P2WPKH },
  { name: 'taproot_electrum', backend: 'electrum', protocol: 'Taproot', addressType: AddressType.P2TR },
]

const runOrSkip = liveTestsEnabled ? test.serial : test.serial.skip

for (const { name, backend, protocol, addressType } of CASES) {
  runOrSkip(name, async (t) => {
    console.log(`\n=== ${name} (${backend} / ${protocol}) ===`)
    cleanupWallet(name)

    const rpcConfig: RpcConfig | undefined =
      backend === 'rpc'
        ? { url: 'localhost:18442', username: 'user', password: 'password', walletName: name }
        : undefined
    const backendConfig: BackendConfig | undefined =
      backend === 'electrum' ? { kind: 'electrum', url: 'tcp://localhost:50001' } : undefined

    const dataDir = path.join(os.homedir(), '.coinswap', 'taker', name)

    const taker = new Taker(
      dataDir,
      name,
      rpcConfig,
      9051,
      'coinswap',
      'tcp://127.0.0.1:28332',
      '',
      backendConfig,
    )

    taker.syncOfferbookAndWait()

    // Fund with 0.25 BTC across 4 fresh external addresses (1.0 BTC total).
    for (let i = 0; i < 4; i += 1) {
      const address = taker.getNextExternalAddress(addressType)
      fund(address.address)
    }
    await sleep(1_000)

    const target = SWAP_AMOUNT * 2
    const funded = await waitForSpendable(taker, target)
    t.true(funded.spendable >= target, `${name}: spendable ${funded.spendable} < target ${target}`)

    const swapId = taker.prepareCoinswap({
      protocol,
      sendAmount: SWAP_AMOUNT,
      makerCount: 2,
      txCount: 1,
      requiredConfirms: 1,
    })
    const report = taker.startCoinswap(swapId)

    t.is(report.makersCount ?? 0, 2, `${name}: should route through 2 makers`)
    t.true(report.status.toUpperCase().includes('SUCCESS'), `${name}: swap status was ${report.status}`)

    console.log(`✓ ${name} passed (swap_id ${report.swapId})`)
  })
}
