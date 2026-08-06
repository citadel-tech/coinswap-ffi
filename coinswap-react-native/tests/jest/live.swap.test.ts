import { AddressType, CoinswapTaker } from '../../src'

import {
  cleanupWallet,
  electrumBackendConfig,
  fundAddress,
  liveTestsEnabled,
  rpcConfig,
  sleep,
  waitForSpendable,
} from './liveTestHelpers'

const describeLive = liveTestsEnabled ? describe : describe.skip

// Sats swapped per taker; funded with 4x this (1.0 BTC across 4 addresses).
const SWAP_AMOUNT = 500_000n

type Backend = 'rpc' | 'electrum'

type SwapCase = {
  name: string
  backend: Backend
  protocol: 'Legacy' | 'Taproot'
  addressType: AddressType
  wallet: string
}

const CASES: SwapCase[] = [
  { name: 'legacy_rpc', backend: 'rpc', protocol: 'Legacy', addressType: AddressType.P2WPKH, wallet: 'rn_legacy_rpc_wallet' },
  { name: 'taproot_rpc', backend: 'rpc', protocol: 'Taproot', addressType: AddressType.P2TR, wallet: 'rn_taproot_rpc_wallet' },
  { name: 'legacy_electrum', backend: 'electrum', protocol: 'Legacy', addressType: AddressType.P2WPKH, wallet: 'rn_legacy_electrum_wallet' },
  { name: 'taproot_electrum', backend: 'electrum', protocol: 'Taproot', addressType: AddressType.P2TR, wallet: 'rn_taproot_electrum_wallet' },
]

async function runSwap({ name, backend, protocol, addressType, wallet }: SwapCase) {
  console.log(`\n=== ${name} (${backend} / ${protocol}) ===`)
  cleanupWallet(wallet)

  await CoinswapTaker.setupLogging(null, 'info')

  const taker = await CoinswapTaker.init({
    dataDir: null,
    walletFileName: wallet,
    rpcConfig: backend === 'rpc' ? rpcConfig(wallet) : null,
    controlPort: 9051,
    torAuthPassword: 'coinswap',
    zmqAddr: 'tcp://127.0.0.1:28332',
    password: '',
    nostrRelays: null,
    backendConfig: backend === 'electrum' ? electrumBackendConfig() : null,
  })

  await taker.syncOfferbookAndWait()

  // Fund with 0.25 BTC across 4 fresh external addresses (1.0 BTC total).
  for (let i = 0; i < 4; i += 1) {
    const address = await taker.getNextExternalAddress(addressType)
    fundAddress(address.addr, '0.25')
  }
  await sleep(1_000)

  const target = SWAP_AMOUNT * 2n
  const funded = await waitForSpendable(taker, target)
  expect(funded.spendable).toBeGreaterThanOrEqual(target)

  const swapId = await taker.prepareCoinswap({
    protocol,
    sendAmount: SWAP_AMOUNT,
    makerCount: 2,
    txCount: 1,
    requiredConfirms: 1,
  })

  const report = await taker.startCoinswap(swapId)
  expect(report.makersCount ?? 0).toBe(2)
  expect(report.status.toUpperCase()).toContain('SUCCESS')

  console.log(`✓ ${name} passed (swap_id ${report.swapId})`)

  await taker.dispose()
}

describeLive('React Native live swap matrix (backend x protocol)', () => {
  for (const testCase of CASES) {
    test(
      testCase.name,
      async () => {
        await runSwap(testCase)
      },
      10 * 60 * 1000,
    )
  }
})
