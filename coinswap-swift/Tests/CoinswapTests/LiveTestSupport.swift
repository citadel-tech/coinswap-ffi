import Foundation
import XCTest
import Coinswap

struct LiveTestConfig {
    let rpcConfig: RpcConfig
    let zmqAddr: String
    let walletName: String
    let dataDir: String?
    let walletPassword: String?
    let torControlPort: UInt16
    let torAuthPassword: String
    let bitcoinNetwork: String
    let dockerContainer: String
    let fundingWallet: String
    let bitcoinRpcPort: String

    init(walletNameOverride: String? = nil) throws {
        let walletName = walletNameOverride ?? "swift_test_wallet"

        self.rpcConfig = RpcConfig(url: "127.0.0.1:18442", username: "user", password: "password", walletName: walletName)
        self.zmqAddr = "tcp://127.0.0.1:28332"
        self.walletName = walletName
        self.dataDir = nil
        self.walletPassword = nil
        self.torControlPort = 9051
        self.torAuthPassword = "coinswap"
        self.bitcoinNetwork = "regtest"
        self.dockerContainer = "coinswap-bitcoind"
        self.fundingWallet = "test"
        self.bitcoinRpcPort = "18442"
    }
}

func requireLiveTestsEnabled() throws {
    let disabled = ProcessInfo.processInfo.environment["COINSWAP_LIVE_TESTS"] == "0"
    if disabled {
        throw XCTSkip("Set COINSWAP_LIVE_TESTS=1 to disable the live tests")
    }
}

/// Backend selection for a live swap run.
enum Backend {
    case rpc
    case electrum
}

/// Electrum backend config pointing at the Docker regtest electrs server.
/// RPC backend is expressed via `RpcConfig` + `backendConfig: nil` instead.
func electrumBackendConfig() -> BackendConfig {
    BackendConfig(
        kind: "electrum",
        url: "tcp://localhost:50001",
        username: nil,
        password: nil,
        walletName: nil,
        zmqAddr: nil,
        socks5: nil,
        timeout: nil,
        pollIntervalSecs: nil,
        maxRetries: nil
    )
}

/// Polls `syncAndSave` + `getBalances` until spendable reaches `target`.
/// Needed because the Electrum backend lags electrs indexing; ~30 tries / 3s.
func waitForSpendable(_ taker: Taker, target: Int64) throws -> Balances {
    for _ in 0..<30 {
        try taker.syncAndSave()
        let balances = try taker.getBalances()
        if balances.spendable >= target {
            return balances
        }
        Thread.sleep(forTimeInterval: 3.0)
    }
    return try taker.getBalances()
}

func runProcess(command: String, args: [String]) throws {
    let process = Process()
    process.executableURL = URL(fileURLWithPath: "/bin/bash")
    let fullCommand = ([command] + args).joined(separator: " ")
    process.arguments = ["-c", fullCommand]

    let pipe = Pipe()
    process.standardOutput = pipe
    process.standardError = pipe

    try process.run()
    process.waitUntilExit()

    if process.terminationStatus != 0 {
        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        let output = String(data: data, encoding: .utf8) ?? ""
        throw NSError(domain: "CoinswapLiveTests", code: Int(process.terminationStatus), userInfo: [
            NSLocalizedDescriptionKey: "Command failed: \(command) \(args.joined(separator: " "))\n\(output)"
        ])
    }
}

/// Cleans up a specific wallet in ~/.coinswap/taker/wallets before running tests.
func cleanupCoinswapData(walletName: String) throws {
    let fileManager = FileManager.default
    let walletPath = URL(fileURLWithPath: NSHomeDirectory())
        .appendingPathComponent(".coinswap/taker/wallets")
        .appendingPathComponent(walletName)

    if fileManager.fileExists(atPath: walletPath.path) {
        try fileManager.removeItem(at: walletPath)
        print("[INFO] Cleaned up wallet: \(walletPath.path)")
    }
}
