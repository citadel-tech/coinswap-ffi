import installer from './NativeCoinswapReactNative'
import coinswapBindings, {
  Taker,
  setupLogging as generatedSetupLogging,
  type Address,
  type Balances,
  type RpcConfig,
  type SwapParams,
  type SwapReport,
  type TakerLike,
} from './generated/coinswap'

export type { RpcConfig, Balances, SwapReport, Address, SwapParams }

export const AddressType = {
  P2WPKH: 'P2WPKH',
  P2TR: 'P2TR',
} as const
export type AddressType = (typeof AddressType)[keyof typeof AddressType]

export type TakerInitConfig = {
  dataDir?: string | null
  walletFileName?: string | null
  rpcConfig?: RpcConfig | null
  controlPort?: number | null
  torAuthPassword?: string | null
  zmqAddr: string
  password?: string | null
}

let rustInstalled = false
let bindingsInitialized = false

function tryInitializeNativeBindings(): boolean {
  if (bindingsInitialized) {
    return true
  }

  try {
    if (!rustInstalled) {
      rustInstalled = installer.installRustCrate()
    }

    if (!rustInstalled) {
      return false
    }

    coinswapBindings.initialize()
    bindingsInitialized = true
    return true
  } catch {
    return false
  }
}

function ensureNativeBindings(): void {
  if (!tryInitializeNativeBindings()) {
    throw new Error('Coinswap native bindings are unavailable. Ensure the TurboModule is installed.')
  }
}

export function isNativeCoinswapAvailable(): boolean {
  return tryInitializeNativeBindings()
}

// This keeps parity with the generated `index.web.ts` API shape.
export async function uniffiInitAsync(): Promise<void> {
  ensureNativeBindings()
}

export class CoinswapTaker {
  private constructor(private readonly taker: TakerLike) {}

  static async setupLogging(
    dataDir: string | null | undefined,
    _level: string,
  ): Promise<void> {
    ensureNativeBindings()
    generatedSetupLogging(dataDir ?? undefined)
  }

  static async init(config: TakerInitConfig): Promise<CoinswapTaker> {
    ensureNativeBindings()

    const taker = Taker.init(
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
    const disposable = this.taker as unknown as { uniffiDestroy?: () => void }
    disposable.uniffiDestroy?.()
  }

  async syncOfferbookAndWait(): Promise<void> {
    this.taker.syncOfferbookAndWait()
  }

  async syncAndSave(): Promise<void> {
    this.taker.syncAndSave()
  }

  async getBalances(): Promise<Balances> {
    return this.taker.getBalances()
  }

  async getNextExternalAddress(addressType: AddressType): Promise<Address> {
    return this.taker.getNextExternalAddress({ addrType: addressType })
  }

  async prepareCoinswap(swapParams: SwapParams): Promise<string> {
    return this.taker.prepareCoinswap(swapParams)
  }

  async startCoinswap(swapId: string): Promise<SwapReport> {
    return this.taker.startCoinswap(swapId)
  }
}
