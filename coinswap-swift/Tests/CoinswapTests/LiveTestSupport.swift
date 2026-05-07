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
    let performSwap: Bool
    let swapAmount: UInt64
    let bitcoinNetwork: String
    let fundingWallet: String
    let bitcoinRpcPort: String
    let fundAmount: String

    init(walletNameOverride: String? = nil) throws {
        let walletName = walletNameOverride ?? "swift_test_wallet"

        self.rpcConfig = RpcConfig(url: "127.0.0.1:18442", username: "user", password: "password", walletName: walletName)
        self.zmqAddr = "tcp://127.0.0.1:28332"
        self.walletName = walletName
        self.dataDir = nil
        self.walletPassword = nil
        self.torControlPort = 9051
        self.torAuthPassword = "coinswap"
        self.performSwap = true
        self.swapAmount = 500000
        self.bitcoinNetwork = "regtest"
        self.fundingWallet = "funding"
        self.bitcoinRpcPort = "18442"
        self.fundAmount = "1.0"
    }
}

func requireLiveTestsEnabled() throws {
    let disabled = ProcessInfo.processInfo.environment["COINSWAP_LIVE_TESTS"] == "0"
    if disabled {
        throw XCTSkip("Live tests are disabled. Set COINSWAP_LIVE_TESTS=1 to enable them.")
    }
}

/// Resolves the absolute path of a command-line tool using `xcrun -f` (which searches PATH
/// correctly on macOS regardless of Apple Silicon vs Intel Homebrew prefix).
private func resolveExecutable(_ name: String) throws -> String {
    let process = Process()
    process.executableURL = URL(fileURLWithPath: "/usr/bin/xcrun")
    process.arguments = ["-f", name]
    let pipe = Pipe()
    process.standardOutput = pipe
    process.standardError = Pipe()
    try process.run()
    process.waitUntilExit()
    guard process.terminationStatus == 0 else {
        throw NSError(domain: "CoinswapLiveTests", code: 1, userInfo: [
            NSLocalizedDescriptionKey: "Could not locate \(name) on PATH via xcrun -f"
        ])
    }
    let data = pipe.fileHandleForReading.readDataToEndOfFile()
    return (String(data: data, encoding: .utf8) ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
}

/// Writes a temporary bitcoin.conf file containing the RPC credentials and returns its path.
/// Using a conf file prevents the username and password from appearing in the process list,
/// where they would be visible to any user on the system via `ps`.
private func writeTempBitcoinConf(config: LiveTestConfig) throws -> URL {
    let confURL = URL(fileURLWithPath: NSTemporaryDirectory())
        .appendingPathComponent("coinswap-test-\(UUID().uuidString).conf")
    let contents = """
    rpcuser=\(config.rpcConfig.username)
    rpcpassword=\(config.rpcConfig.password)
    """
    try contents.write(to: confURL, atomically: true, encoding: .utf8)
    return confURL
}

/// Sends `config.fundAmount` BTC to `address` on the regtest node, then mines one block
/// to confirm the transaction so the funds are immediately spendable by the Taker wallet.
func fundAddress(_ address: String, config: LiveTestConfig) throws {
    let bitcoinCLI = try resolveExecutable("bitcoin-cli")
    let confURL = try writeTempBitcoinConf(config: config)
    defer { try? FileManager.default.removeItem(at: confURL) }

    // Send funds to the taker address.
    try runProcess(executablePath: bitcoinCLI, args: [
        "-regtest",
        "-rpcport=\(config.bitcoinRpcPort)",
        "-conf=\(confURL.path)",
        "-rpcwallet=\(config.fundingWallet)",
        "sendtoaddress",
        address,
        config.fundAmount
    ])

    // Mine one block so the transaction is confirmed and the UTXO is spendable.
    // Without this step the Taker wallet sees the funds as unconfirmed and cannot
    // select them as inputs for the swap funding transaction.
    let miningAddress = try bitcoinCliOutput(executablePath: bitcoinCLI, args: [
        "-regtest",
        "-rpcport=\(config.bitcoinRpcPort)",
        "-conf=\(confURL.path)",
        "-rpcwallet=\(config.fundingWallet)",
        "getnewaddress",
        "",
        "bech32m"
    ])
    try runProcess(executablePath: bitcoinCLI, args: [
        "-regtest",
        "-rpcport=\(config.bitcoinRpcPort)",
        "-conf=\(confURL.path)",
        "-rpcwallet=\(config.fundingWallet)",
        "generatetoaddress",
        "1",
        miningAddress.trimmingCharacters(in: .whitespacesAndNewlines)
    ])
}

/// Runs an external process by calling the binary directly — no shell string is constructed,
/// so shell metacharacters in any argument cannot be interpreted as commands.
func runProcess(executablePath: String, args: [String]) throws {
    let process = Process()
    process.executableURL = URL(fileURLWithPath: executablePath)
    process.arguments = args

    let pipe = Pipe()
    process.standardOutput = pipe
    process.standardError = pipe

    try process.run()
    process.waitUntilExit()

    if process.terminationStatus != 0 {
        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        let output = String(data: data, encoding: .utf8) ?? ""
        throw NSError(domain: "CoinswapLiveTests", code: Int(process.terminationStatus), userInfo: [
            NSLocalizedDescriptionKey: "Command failed: \(executablePath) \(args.joined(separator: " "))\n\(output)"
        ])
    }
}

/// Runs an external process and returns its standard output as a String.
private func bitcoinCliOutput(executablePath: String, args: [String]) throws -> String {
    let process = Process()
    process.executableURL = URL(fileURLWithPath: executablePath)
    process.arguments = args

    let pipe = Pipe()
    process.standardOutput = pipe
    process.standardError = Pipe()

    try process.run()
    process.waitUntilExit()

    if process.terminationStatus != 0 {
        throw NSError(domain: "CoinswapLiveTests", code: Int(process.terminationStatus), userInfo: [
            NSLocalizedDescriptionKey: "Command failed: \(executablePath) \(args.joined(separator: " "))"
        ])
    }
    let data = pipe.fileHandleForReading.readDataToEndOfFile()
    return String(data: data, encoding: .utf8) ?? ""
}

/// Polls the Taker wallet balance until at least one confirmed UTXO is present, or the
/// deadline is reached. This replaces a fixed `Thread.sleep` which is non-deterministic:
/// too short causes flaky failures on a slow node; too long wastes time on a fast one.
func waitForConfirmedBalance(taker: Taker, timeoutSeconds: Double = 30.0) throws {
    let deadline = Date().addingTimeInterval(timeoutSeconds)
    while Date() < deadline {
        try taker.syncAndSave()
        let balances = try taker.getBalances()
        if balances.spendable > 0 {
            return
        }
        Thread.sleep(forTimeInterval: 1.0)
    }
    throw NSError(domain: "CoinswapLiveTests", code: 1, userInfo: [
        NSLocalizedDescriptionKey: "Taker wallet balance did not become spendable within \(Int(timeoutSeconds))s"
    ])
}

/// Asserts that an Int64 value is within ±tolerance of the expected value.
func assertApprox(_ actual: Int64, _ expected: Int64, tolerance: Int64 = 2,
                  file: StaticString = #filePath, line: UInt = #line) {
    XCTAssertGreaterThanOrEqual(actual, expected - tolerance,
        "Value \(actual) below expected range [\(expected - tolerance)...\(expected + tolerance)]",
        file: file, line: line)
    XCTAssertLessThanOrEqual(actual, expected + tolerance,
        "Value \(actual) above expected range [\(expected - tolerance)...\(expected + tolerance)]",
        file: file, line: line)
}

/// Asserts that a Double value is within ±tolerance of the expected value.
func assertApprox(_ actual: Double, _ expected: Double, tolerance: Double = 2.0,
                  file: StaticString = #filePath, line: UInt = #line) {
    XCTAssertGreaterThanOrEqual(actual, expected - tolerance,
        "Value \(actual) below expected range [\(expected - tolerance)...\(expected + tolerance)]",
        file: file, line: line)
    XCTAssertLessThanOrEqual(actual, expected + tolerance,
        "Value \(actual) above expected range [\(expected - tolerance)...\(expected + tolerance)]",
        file: file, line: line)
}

/// Removes wallet state and stale Nostr cursor data so each test run starts clean.
///
/// - Deletes the named wallet directory so a fresh wallet is created.
/// - Deletes `.taker_watcher/` which holds CBOR-encoded per-relay Nostr cursors.
///   Without this, the taker subscribes with `since=<last run timestamp>` and the
///   relay returns no events, leaving the offerbook empty.
/// - Deletes `offerbook.json` so the offerbook is rebuilt from the fresh subscription.
/// Each deletion is attempted independently; a failure to remove one item logs a warning
/// rather than aborting the entire setup, so a partial cleanup does not block the test.
func cleanupCoinswapData(walletName: String) {
    let fileManager = FileManager.default
    let takerDir = URL(fileURLWithPath: NSHomeDirectory()).appendingPathComponent(".coinswap/taker")

    let walletPath = takerDir.appendingPathComponent("wallets").appendingPathComponent(walletName)
    if fileManager.fileExists(atPath: walletPath.path) {
        do {
            try fileManager.removeItem(at: walletPath)
            print("[INFO] Cleaned up wallet: \(walletPath.path)")
        } catch {
            print("[WARN] Could not remove wallet at \(walletPath.path): \(error)")
        }
    }

    // Remove stale Nostr cursor registry so the taker fetches all maker events.
    let watcherDir = takerDir.appendingPathComponent(".taker_watcher")
    if fileManager.fileExists(atPath: watcherDir.path) {
        do {
            try fileManager.removeItem(at: watcherDir)
        } catch {
            print("[WARN] Could not remove watcher dir at \(watcherDir.path): \(error)")
        }
    }

    // Remove cached offerbook so it is rebuilt from scratch.
    let offerbookPath = takerDir.appendingPathComponent("offerbook.json")
    if fileManager.fileExists(atPath: offerbookPath.path) {
        do {
            try fileManager.removeItem(at: offerbookPath)
        } catch {
            print("[WARN] Could not remove offerbook at \(offerbookPath.path): \(error)")
        }
    }
}
