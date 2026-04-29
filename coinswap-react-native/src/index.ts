import {
  getNativeCoinswapModule,
  isNativeCoinswapAvailable,
  type Address,
  type Balances,
  type RpcConfig,
  type SwapParams,
  type SwapReport,
  type TakerInitConfig,
} from './NativeCoinswap'

export { isNativeCoinswapAvailable }
export type { Address, Balances, RpcConfig, SwapParams, SwapReport, TakerInitConfig }

export const AddressType = {
  P2WPKH: 'P2WPKH',
  P2TR: 'P2TR',
} as const

export type AddressType = (typeof AddressType)[keyof typeof AddressType]

export class CoinswapTaker {
  private readonly takerId: string

  private constructor(takerId: string) {
    this.takerId = takerId
  }

  static async setupLogging(dataDir: string | null, level: string): Promise<void> {
    await getNativeCoinswapModule().setupLogging(dataDir, level)
  }

  static async init(config: TakerInitConfig): Promise<CoinswapTaker> {
    const takerId = await getNativeCoinswapModule().createTaker(config)
    return new CoinswapTaker(takerId)
  }

  async dispose(): Promise<void> {
    await getNativeCoinswapModule().disposeTaker(this.takerId)
  }

  async syncOfferbookAndWait(): Promise<void> {
    await getNativeCoinswapModule().syncOfferbookAndWait(this.takerId)
  }

  async syncAndSave(): Promise<void> {
    await getNativeCoinswapModule().syncAndSave(this.takerId)
  }

  async getBalances(): Promise<Balances> {
    return getNativeCoinswapModule().getBalances(this.takerId)
  }

  async getNextExternalAddress(addressType: AddressType): Promise<Address> {
    return getNativeCoinswapModule().getNextExternalAddress(this.takerId, addressType)
  }

  async prepareCoinswap(swapParams: SwapParams): Promise<string> {
    return getNativeCoinswapModule().prepareCoinswap(this.takerId, swapParams)
  }

  async startCoinswap(swapId: string): Promise<SwapReport> {
    return getNativeCoinswapModule().startCoinswap(this.takerId, swapId)
  }
}
