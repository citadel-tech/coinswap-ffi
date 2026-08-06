import Foundation
import XCTest
import Coinswap

/// Consolidated live swap suite covering the full backend x protocol matrix.
///
/// Runs against the Docker regtest stack (1 RPC maker + 1 Electrum maker). Each
/// taker: init -> fund (0.25 BTC x 4 fresh external addresses = 1.0 BTC) ->
/// sync until funds are visible -> 2-maker coinswap -> assert the report routed
/// through 2 makers and the status reports SUCCESS.
final class LiveSwapTests: XCTestCase {
    /// Sats swapped per taker (funded with 1.0 BTC, well above this).
    private let swapAmount: UInt64 = 500_000

    func testLegacyRpc() throws {
        try runSwap(name: "legacyRpc", backend: .rpc, protocol: "Legacy",
                    addrType: "P2WPKH", wallet: "swift_legacy_rpc_wallet")
    }

    func testTaprootRpc() throws {
        try runSwap(name: "taprootRpc", backend: .rpc, protocol: "Taproot",
                    addrType: "P2TR", wallet: "swift_taproot_rpc_wallet")
    }

    func testLegacyElectrum() throws {
        try runSwap(name: "legacyElectrum", backend: .electrum, protocol: "Legacy",
                    addrType: "P2WPKH", wallet: "swift_legacy_electrum_wallet")
    }

    func testTaprootElectrum() throws {
        try runSwap(name: "taprootElectrum", backend: .electrum, protocol: "Taproot",
                    addrType: "P2TR", wallet: "swift_taproot_electrum_wallet")
    }

    /// Runs one taker end-to-end for the given backend/protocol/address type.
    private func runSwap(
        name: String,
        backend: Backend,
        protocol proto: String,
        addrType: String,
        wallet: String
    ) throws {
        try requireLiveTestsEnabled()
        print("\n=== \(name) (\(backend) / \(proto) / \(addrType)) ===")
        try cleanupCoinswapData(walletName: wallet)

        let config = try LiveTestConfig(walletNameOverride: wallet)

        // RPC backend: RpcConfig + backendConfig nil.
        // Electrum backend: rpcConfig nil + electrum BackendConfig.
        let rpcConfig: RpcConfig? = backend == .rpc ? config.rpcConfig : nil
        let backendConfig: BackendConfig? = backend == .electrum ? electrumBackendConfig() : nil

        let taker = try Taker.`init`(
            dataDir: config.dataDir,
            walletFileName: config.walletName,
            rpcConfig: rpcConfig,
            controlPort: config.torControlPort,
            torAuthPassword: config.torAuthPassword,
            zmqAddr: config.zmqAddr,
            password: config.walletPassword,
            nostrRelays: nil,
            backendConfig: backendConfig
        )

        try taker.setupLogging(dataDir: config.dataDir, logLevel: "Info")
        try taker.syncOfferbookAndWait()
        XCTAssertEqual(try taker.getWalletName(), config.walletName)

        // Fund with 0.25 BTC across 4 fresh external addresses (1.0 BTC total),
        // then wait for the balance to become spendable (tolerates Electrum lag).
        try fundFreshAddresses(taker, addrType: addrType, config: config)
        let funded = try waitForSpendable(taker, target: Int64(swapAmount))
        XCTAssertGreaterThanOrEqual(
            funded.spendable, Int64(swapAmount),
            "\(name): spendable should cover the swap amount")

        // 2-maker coinswap, single funding tx, 1 required confirmation.
        let params = SwapParams(
            protocol: proto,
            sendAmount: swapAmount,
            makerCount: 2,
            txCount: 1,
            requiredConfirms: 1,
            manuallySelectedOutpoints: nil,
            preferredMakers: nil
        )
        let swapId = try taker.prepareCoinswap(swapParams: params)
        let report = try taker.startCoinswap(swapId: swapId)

        XCTAssertEqual(
            report.makersCount, 2,
            "\(name): swap should route through 2 makers")
        // `status` is a display string (may carry ANSI color); match on content.
        XCTAssertTrue(
            report.status.uppercased().contains("SUCCESS"),
            "\(name): swap status was \(report.status)")

        print("✓ \(name) passed (swap_id \(report.swapId))")
        fflush(stdout)
    }

    /// Funds `taker` with 0.25 BTC across 4 fresh external addresses.
    private func fundFreshAddresses(
        _ taker: Taker, addrType: String, config: LiveTestConfig
    ) throws {
        for _ in 0..<4 {
            let addr = try taker.getNextExternalAddress(
                addressType: AddressType(addrType: addrType)
            ).addr
            try runProcess(command: "docker", args: [
                "exec", config.dockerContainer, "bitcoin-cli",
                "-\(config.bitcoinNetwork)",
                "-rpcport=\(config.bitcoinRpcPort)",
                "-rpcwallet=\(config.fundingWallet)",
                "-rpcuser=user", "-rpcpassword=password",
                "sendtoaddress", addr, "0.25",
            ])
        }
        Thread.sleep(forTimeInterval: 1.0)
    }
}
