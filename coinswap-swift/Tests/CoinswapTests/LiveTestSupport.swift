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
    let dockerContainer: String
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
        self.dockerContainer = "coinswap-bitcoind"
        self.fundingWallet = "test"
        self.bitcoinRpcPort = "18442"
        self.fundAmount = "1.0"
    }
}

func requireLiveTestsEnabled() throws {
    let disabled = ProcessInfo.processInfo.environment["COINSWAP_LIVE_TESTS"] == "0"
    if disabled {
        throw XCTSkip("Set COINSWAP_LIVE_TESTS=1 to disable the live tests")
    }
}

func fundAddress(_ address: String, config: LiveTestConfig) throws {
    let args: [String] = [
        "exec",
        "coinswap-bitcoind",
        "bitcoin-cli",
        "-regtest",
        "-rpcport=18442",
        "-rpcwallet=test",
        "-rpcuser=user",
        "-rpcpassword=password",
        "sendtoaddress",
        address,
        "1.0"
    ]

    try runProcess(command: "docker", args: args)
    Thread.sleep(forTimeInterval: 1.0)
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
