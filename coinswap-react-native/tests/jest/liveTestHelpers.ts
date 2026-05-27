import { execFileSync } from 'node:child_process'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'

export const liveTestsEnabled = process.env.COINSWAP_LIVE_TESTS === '1'

const RPC_AUTH_ARGS = ['-regtest', '-rpcport=18442', '-rpcuser=user', '-rpcpassword=password']

function dockerContainers(): string[] {
  const envContainer = process.env.COINSWAP_DOCKER_CONTAINER?.trim()
  const values = [envContainer, 'coinswap-bitcoind', 'coinswap-ffi-bitcoind'].filter(
    (value): value is string => Boolean(value),
  )

  return [...new Set(values)]
}

function runDockerExec(container: string, args: string[]): string {
  return execFileSync('docker', ['exec', container, ...args], {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  }).trim()
}

function runBitcoinCli(args: string[]): string {
  let lastError: Error | undefined

  for (const container of dockerContainers()) {
    try {
      return runDockerExec(container, ['bitcoin-cli', ...args])
    } catch (error) {
      lastError = error as Error
    }
  }

  throw new Error(`Failed to execute bitcoin-cli in Docker: ${lastError?.message ?? 'unknown error'}`)
}

export function cleanupWallet(walletName: string) {
  const walletsDir = path.join(os.homedir(), '.coinswap', 'taker', 'wallets')

  if (fs.existsSync(walletsDir)) {
    for (const entry of fs.readdirSync(walletsDir)) {
      if (!entry.startsWith(walletName)) {
        continue
      }
      fs.rmSync(path.join(walletsDir, entry), { recursive: true, force: true })
    }
  }

  try {
    runBitcoinCli([...RPC_AUTH_ARGS, 'unloadwallet', walletName])
  } catch {
    // Ignore missing wallet errors.
  }

  for (const container of dockerContainers()) {
    try {
      runDockerExec(container, ['rm', '-rf', `/home/bitcoin/.bitcoin/wallets/${walletName}`])
      return
    } catch {
      // Try next container.
    }
  }
}

export function fundAddress(address: string, amountBtc: string) {
  return runBitcoinCli([...RPC_AUTH_ARGS, '-rpcwallet=test', 'sendtoaddress', address, amountBtc])
}

export function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}
