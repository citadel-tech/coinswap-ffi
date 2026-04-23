import { AddressType, CoinswapTaker, isNativeCoinswapAvailable } from '../src'

import { cleanupWallet, fundAddress, liveTestsEnabled, sleep } from './liveTestHelpers'

const describeLive = liveTestsEnabled ? describe : describe.skip

describeLive('React Native live standard swap (legacy)', () => {
  const walletName = 'rn_legacy_wallet'

  beforeAll(() => {
    if (!isNativeCoinswapAvailable()) {
      throw new Error('Coinswap TurboModule is unavailable in this runtime')
    }
  })

  test(
    'runs end-to-end legacy swap',
    async () => {
      cleanupWallet(walletName)

      await CoinswapTaker.setupLogging(null, 'info')

      const taker = await CoinswapTaker.init({
        dataDir: null,
        walletFileName: walletName,
        rpcConfig: {
          url: 'localhost:18442',
          username: 'user',
          password: 'password',
          walletName,
        },
        controlPort: 9051,
        torAuthPassword: 'coinswap',
        zmqAddr: 'tcp://127.0.0.1:28332',
        password: '',
      })

      await taker.syncOfferbookAndWait()
      await taker.syncAndSave()

      const address = await taker.getNextExternalAddress(AddressType.P2WPKH)
      expect(address.address).toBeTruthy()

      fundAddress(address.address, '1.0')
      await sleep(1_000)
      await taker.syncAndSave()

      const balances = await taker.getBalances()
      expect(balances.spendable).toBeGreaterThan(0)

      const swapId = await taker.prepareCoinswap({
        protocol: 'Legacy',
        sendAmount: 500_000,
        makerCount: 2,
        txCount: 1,
        requiredConfirms: 1,
      })

      const report = await taker.startCoinswap(swapId)
      expect(report.swapId).toBe(swapId)
      expect(report.outgoingAmount).toBe(500_000)
      expect(report.makersCount).toBeGreaterThanOrEqual(2)
      expect(report.makerAddresses.length).toBeGreaterThanOrEqual(2)

      await taker.dispose()
    },
    10 * 60 * 1000,
  )
})
