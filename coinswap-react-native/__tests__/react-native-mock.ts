// Jest-only mock to avoid loading the real React Native runtime in Node tests.
let takerCount = 0
let swapCount = 0

type SwapParams = {
  protocol?: 'Legacy' | 'Taproot'
  sendAmount: number
  makerCount: number
  txCount?: number
  requiredConfirms?: number
  preferredMakers?: string[]
}

type SwapReport = {
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

type Balances = {
  regular: number
  swap: number
  contract: number
  fidelity: number
  spendable: number
}

type TakerState = {
  balances: Balances
  lastSwap?: {
    swapId: string
    params: SwapParams
  }
}

const takers = new Map<string, TakerState>()

function defaultBalances(): Balances {
  return {
    regular: 0,
    swap: 0,
    contract: 0,
    fidelity: 0,
    spendable: 1_000_000,
  }
}

function makeReport(swapId: string, params: SwapParams): SwapReport {
  const makersCount = Math.max(params.makerCount ?? 2, 2)
  const makerAddresses = Array.from({ length: makersCount }, (_, index) => `maker-${index + 1}`)

  return {
    swapId,
    outgoingAmount: params.sendAmount,
    incomingAmount: params.sendAmount,
    feePaidOrEarned: 0,
    makersCount,
    makerAddresses,
    totalMakerFees: 0,
    miningFee: 0,
    status: 'Completed',
  }
}

const coinswapModule = {
  async setupLogging() {
    return
  },
  async createTaker() {
    const takerId = `taker-${++takerCount}`
    takers.set(takerId, { balances: defaultBalances() })
    return takerId
  },
  async disposeTaker(takerId: string) {
    takers.delete(takerId)
  },
  async syncOfferbookAndWait() {
    return
  },
  async syncAndSave() {
    return
  },
  async getBalances(takerId: string) {
    return takers.get(takerId)?.balances ?? defaultBalances()
  },
  async getNextExternalAddress() {
    return { address: 'bcrt1qjestonlyaddress0000000000000000000' }
  },
  async prepareCoinswap(takerId: string, params: SwapParams) {
    const swapId = `swap-${++swapCount}`
    const state = takers.get(takerId)
    if (state) {
      state.lastSwap = { swapId, params }
    }
    return swapId
  },
  async startCoinswap(takerId: string, swapId: string) {
    const state = takers.get(takerId)
    const params = state?.lastSwap?.params ?? { sendAmount: 500_000, makerCount: 2 }
    return makeReport(swapId, params)
  },
}

export const TurboModuleRegistry = {
  get: (moduleName: string) => (moduleName === 'Coinswap' ? coinswapModule : null),
}
