// Run `npm run ubrn:generate` to populate src/generated/ with real bindings.
// The stub at src/generated/coinswap.ts keeps TypeScript happy until then.
import { Taker, AddressType as GeneratedAddressType } from './generated/coinswap'
export type { RpcConfig, Balances, SwapReport, Address, SwapParams } from './generated/coinswap'

// Our public AddressType constants — converted to a GeneratedAddressType instance at call sites.
export const AddressType = {
  P2WPKH: 'P2WPKH',
  P2TR: 'P2TR',
} as const
export type AddressType = (typeof AddressType)[keyof typeof AddressType]

export type TakerInitConfig = {
  dataDir?: string | null
  walletFileName?: string | null
  rpcConfig?: import('./generated/coinswap').RpcConfig | null
  controlPort?: number | null
  torAuthPassword?: string | null
  zmqAddr: string
  password?: string | null
}

export function isNativeCoinswapAvailable(): boolean {
  try {
    return typeof Taker !== 'undefined'
  } catch {
    return false
  }
}

// CoinswapTaker wraps the generated Taker, preserving the existing config-object init API.
export class CoinswapTaker {
  private constructor(private readonly taker: Taker) {}

  static async setupLogging(dataDir: string | null | undefined, level: string): Promise<void> {
    await Taker.setupLogging(dataDir ?? undefined, level)
  }

  static async init(config: TakerInitConfig): Promise<CoinswapTaker> {
    const taker = await Taker.init(
      config.dataDir ?? undefined,
      config.walletFileName ?? undefined,
      config.rpcConfig ?? undefined,
      config.controlPort ?? undefined,
      config.torAuthPassword ?? undefined,
      config.zmqAddr,
      config.password ?? undefined,
    )
    return new CoinswapTaker(taker)
  }

  async dispose(): Promise<void> {
    this.taker[Symbol.dispose]?.()
  }

  async syncOfferbookAndWait(): Promise<void> {
    await this.taker.syncOfferbookAndWait()
  }

  async syncAndSave(): Promise<void> {
    await this.taker.syncAndSave()
  }

  async getBalances(): Promise<import('./generated/coinswap').Balances> {
    return this.taker.getBalances()
  }

  async getNextExternalAddress(
    addressType: AddressType,
  ): Promise<import('./generated/coinswap').Address> {
    return this.taker.getNextExternalAddress(new GeneratedAddressType(addressType))
  }

  async prepareCoinswap(
    swapParams: import('./generated/coinswap').SwapParams,
  ): Promise<string> {
    return this.taker.prepareCoinswap(swapParams)
  }

  async startCoinswap(swapId: string): Promise<import('./generated/coinswap').SwapReport> {
    return this.taker.startCoinswap(swapId)
  }
}
