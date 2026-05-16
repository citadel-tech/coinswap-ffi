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
        self.fundingWallet = ProcessInfo.processInfo.environment["COINSWAP_FUNDING_WALLET"] ?? "test"
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

// MARK: - Process Helpers

/// Runs a process and blocks until it exits, then returns its exit code, stdout, and stderr.
///
/// Both stdout and stderr are drained **concurrently** on background queues before
/// `waitUntilExit()` returns. This prevents pipe-buffer deadlocks: if we read
/// sequentially and one pipe fills up (~64 KB on macOS), the child blocks on
/// `write(2)` and can never exit, while the parent waits forever on the other
/// pipe's `readDataToEndOfFile()`.
private func runCapture(executablePath: String, args: [String]) throws -> (exitCode: Int32, stdout: String, stderr: String) {
    let process = Process()
    process.executableURL = URL(fileURLWithPath: executablePath)
    process.arguments = args

    let stdoutPipe = Pipe()
    let stderrPipe = Pipe()
    process.standardOutput = stdoutPipe
    process.standardError = stderrPipe

    try process.run()

    let group = DispatchGroup()
    var stdoutData = Data()
    var stderrData = Data()

    group.enter()
    DispatchQueue.global().async {
        stdoutData = stdoutPipe.fileHandleForReading.readDataToEndOfFile()
        group.leave()
    }

    group.enter()
    DispatchQueue.global().async {
        stderrData = stderrPipe.fileHandleForReading.readDataToEndOfFile()
        group.leave()
    }

    group.wait()
    process.waitUntilExit()

    return (
        exitCode: process.terminationStatus,
        stdout: String(data: stdoutData, encoding: .utf8) ?? "",
        stderr: String(data: stderrData, encoding: .utf8) ?? ""
    )
}

private func formatProcessError(executablePath: String, args: [String],
                                exitCode: Int32, stdout: String, stderr: String) -> NSError {
    let body = [stdout, stderr].filter { !$0.isEmpty }.joined(separator: "\n")
    let message = body.isEmpty
        ? "Command failed: \(executablePath) \(args.joined(separator: " ")) (exit code \(exitCode))"
        : "Command failed: \(executablePath) \(args.joined(separator: " "))\n\(body)"
    return NSError(domain: "CoinswapLiveTests", code: Int(exitCode), userInfo: [
        NSLocalizedDescriptionKey: message
    ])
}

/// Resolves the absolute path of a command-line tool.
///
/// First tries `xcrun -f` (which covers Xcode toolchain paths as well as the
/// system PATH on Intel and Apple Silicon runners). Falls back to checking
/// common Homebrew prefixes so tools like `docker` (installed by Colima to
/// `/usr/local/bin`) can be found even when `xcrun -f` fails.
private func resolveExecutable(_ name: String) throws -> String {
    let (exitCode, stdout, _) = try runCapture(executablePath: "/usr/bin/xcrun", args: ["-f", name])
    if exitCode == 0 {
        let resolvedPath = stdout.trimmingCharacters(in: .whitespacesAndNewlines)
        if !resolvedPath.isEmpty {
            return resolvedPath
        }
    }

    // Fallback: check common Homebrew/Colima prefixes.
    let homebrewPrefixes = [
        "/usr/local/bin",
        "/opt/homebrew/bin",
        "/opt/local/bin",
        "/usr/local/libexec",
    ]
    for prefix in homebrewPrefixes {
        let candidate = "\(prefix)/\(name)"
        if FileManager.default.fileExists(atPath: candidate) {
            return candidate
        }
    }

    let detail = [stdout].filter { !$0.isEmpty }.joined(separator: "\n")
    let msg = detail.isEmpty
        ? "Could not locate \(name). Install it and ensure it is on PATH."
        : "Could not locate \(name):\n\(detail)"
    throw NSError(domain: "CoinswapLiveTests", code: 1, userInfo: [
        NSLocalizedDescriptionKey: msg
    ])
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
    try FileManager.default.setAttributes([.posixPermissions: 0o600], ofItemAtPath: confURL.path)
    return confURL
}

/// Sends `config.fundAmount` BTC to `address` on the regtest node running inside the
/// `coinswap-bitcoind` Docker container, then mines one block to confirm the transaction
/// so the funds are immediately spendable by the Taker wallet.
///
/// Uses `docker exec` because the CI runner does not have `bitcoin-cli` installed on the
/// host PATH — Bitcoin Core lives exclusively inside the container.
func fundAddress(_ address: String, config: LiveTestConfig) throws {
    let dockerPath = try resolveExecutable("docker")
    let confURL = try writeTempBitcoinConf(config: config)
    defer { try? FileManager.default.removeItem(at: confURL) }

    func dockerBitcoinCli(_ subArgs: [String]) throws {
        try runProcess(executablePath: dockerPath, args: [
            "exec", "coinswap-bitcoind", "bitcoin-cli",
            "-regtest",
            "-rpcport=\(config.bitcoinRpcPort)",
            "-conf=/tmp/bitcoin.conf",
            "-rpcwallet=\(config.fundingWallet)",
        ] + subArgs)
    }

    func dockerBitcoinCliOutput(_ subArgs: [String]) throws -> String {
        return try bitcoinCliOutput(executablePath: dockerPath, args: [
            "exec", "coinswap-bitcoind", "bitcoin-cli",
            "-regtest",
            "-rpcport=\(config.bitcoinRpcPort)",
            "-conf=/tmp/bitcoin.conf",
            "-rpcwallet=\(config.fundingWallet)",
        ] + subArgs)
    }

    // Copy the temp bitcoin.conf into the container so bitcoin-cli can read it.
    try runProcess(executablePath: dockerPath, args: [
        "cp", confURL.path, "coinswap-bitcoind:/tmp/bitcoin.conf",
    ])

    // Send funds to the taker address.
    try dockerBitcoinCli(["sendtoaddress", address, config.fundAmount])

    // Mine one block so the transaction is confirmed and the UTXO is spendable.
    // Without this step the Taker wallet sees the funds as unconfirmed and cannot
    // select them as inputs for the swap funding transaction.
    let miningAddress = try dockerBitcoinCliOutput([
        "getnewaddress", "", "bech32m",
    ])
    try dockerBitcoinCli([
        "generatetoaddress", "1",
        miningAddress.trimmingCharacters(in: .whitespacesAndNewlines),
    ])
}

/// Runs an external process by calling the binary directly — no shell string is constructed,
/// so shell metacharacters in any argument cannot be interpreted as commands.
func runProcess(executablePath: String, args: [String]) throws {
    let (exitCode, stdout, stderr) = try runCapture(executablePath: executablePath, args: args)
    if exitCode != 0 {
        throw formatProcessError(executablePath: executablePath, args: args,
                                 exitCode: exitCode, stdout: stdout, stderr: stderr)
    }
}

/// Runs an external process and returns its standard output as a String.
private func bitcoinCliOutput(executablePath: String, args: [String]) throws -> String {
    let (exitCode, stdout, stderr) = try runCapture(executablePath: executablePath, args: args)
    if exitCode != 0 {
        throw formatProcessError(executablePath: executablePath, args: args,
                                 exitCode: exitCode, stdout: stdout, stderr: stderr)
    }
    return stdout
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

/// Waits until the local offerbook contains at least `minimumMakers` entries.
///
/// `syncOfferbookAndWait()` only guarantees the discovery cycle has completed once;
/// it does not guarantee the local snapshot already contains enough makers for a
/// two-hop swap. Polling the snapshot keeps the tests from racing the discovery loop.
func waitForOfferbookMakers(
    taker: Taker,
    minimumMakers: Int = 2,
    protocolName: String? = nil,
    timeoutSeconds: Double = 60.0
) throws -> OfferBook {
    let isMatchingMaker: (MakerOfferCandidate) -> Bool = { candidate in
        guard candidate.state.stateType == "Good", candidate.offer != nil else {
            return false
        }
        guard let protocolName else { return true }
        let makerProtocol = candidate.protocol?.protocolType
        return makerProtocol == protocolName || makerProtocol == "Unified"
    }

    let deadline = Date().addingTimeInterval(timeoutSeconds)
    var lastOfferbook: OfferBook?

    while Date() < deadline {
        try taker.syncOfferbookAndWait()
        let offerbook = try taker.fetchOffers()
        lastOfferbook = offerbook
        if offerbook.makers.filter(isMatchingMaker).count >= minimumMakers {
            return offerbook
        }
        Thread.sleep(forTimeInterval: 2.0)
    }

    let observedCount = lastOfferbook?.makers.count ?? 0
    let observedMatchingCount = lastOfferbook?.makers.filter(isMatchingMaker).count ?? 0
    throw NSError(domain: "CoinswapLiveTests", code: 1, userInfo: [
        NSLocalizedDescriptionKey: "Offerbook did not reach \(minimumMakers) good makers\(protocolName.map { " for \($0)" } ?? "") within \(Int(timeoutSeconds))s (last observed total: \(observedCount), matching: \(observedMatchingCount))"
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
///
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
            print("[INFO] Cleaned up watcher dir: \(watcherDir.path)")
        } catch {
            print("[WARN] Could not remove watcher dir at \(watcherDir.path): \(error)")
        }
    }

    // Remove cached offerbook so it is rebuilt from scratch.
    let offerbookPath = takerDir.appendingPathComponent("offerbook.json")
    if fileManager.fileExists(atPath: offerbookPath.path) {
        do {
            try fileManager.removeItem(at: offerbookPath)
            print("[INFO] Cleaned up offerbook: \(offerbookPath.path)")
        } catch {
            print("[WARN] Could not remove offerbook at \(offerbookPath.path): \(error)")
        }
    }
}
