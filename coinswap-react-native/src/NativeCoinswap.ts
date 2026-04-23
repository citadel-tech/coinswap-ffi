import type { TurboModule } from 'react-native'
import { TurboModuleRegistry } from 'react-native'

export type RpcConfig = {
  url: string
  username: string
  password: string
  walletName: string
}

export type TakerInitConfig = {
  dataDir?: string | null
  walletFileName?: string | null
  rpcConfig?: RpcConfig | null
  controlPort?: number | null
  torAuthPassword?: string | null
  zmqAddr: string
  password?: string | null
}

export type SwapParams = {
  protocol?: 'Legacy' | 'Taproot'
  sendAmount: number
  makerCount: number
  txCount?: number
  requiredConfirms?: number
  preferredMakers?: string[]
}

export type Balances = {
  regular: number
  swap: number
  contract: number
  fidelity: number
  spendable: number
}

export type Address = {
  address: string
}

export type SwapReport = {
  swapId: string
  outgoingAmount: number
  incomingAmount: number
  feePaidOrEarned: number
  makersCount: number
  makerAddresses: string[]
  totalMakerFees: number
  miningFee: number
  status: string
}

export interface Spec extends TurboModule {
  setupLogging(dataDir: string | null, level: string): Promise<void>
  createTaker(config: TakerInitConfig): Promise<string>
  disposeTaker(takerId: string): Promise<void>
  syncOfferbookAndWait(takerId: string): Promise<void>
  syncAndSave(takerId: string): Promise<void>
  getBalances(takerId: string): Promise<Balances>
  getNextExternalAddress(takerId: string, addressType: string): Promise<Address>
  prepareCoinswap(takerId: string, swapParams: SwapParams): Promise<string>
  startCoinswap(takerId: string, swapId: string): Promise<SwapReport>
}

let nativeModule: Spec | null = null

function resolveNativeModule(): Spec {
  if (nativeModule) {
    return nativeModule
  }

  const resolved = TurboModuleRegistry.get<Spec>('Coinswap')
  if (!resolved) {
    throw new Error('Coinswap TurboModule is not available in this React Native runtime')
  }

  nativeModule = resolved
  return resolved
}

export function isNativeCoinswapAvailable(): boolean {
  try {
    resolveNativeModule()
    return true
  } catch {
    return false
  }
}

export function getNativeCoinswapModule(): Spec {
  return resolveNativeModule()
}
