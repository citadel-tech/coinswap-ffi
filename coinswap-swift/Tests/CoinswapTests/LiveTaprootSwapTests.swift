import Foundation
import XCTest
import Coinswap

final class LiveTaprootSwapTests: XCTestCase {
    func testLiveTaprootTakerFlow() throws {
        try requireLiveTestsEnabled()
        try cleanupCoinswapData(walletName: "swift_taproot_wallet")
        let config = try LiveTestConfig(walletNameOverride: "swift_taproot_wallet")

        let taker = try Taker.`init`(
            dataDir: config.dataDir,
            walletFileName: config.walletName,
            rpcConfig: config.rpcConfig,
            controlPort: config.torControlPort,
            torAuthPassword: config.torAuthPassword,
            zmqAddr: config.zmqAddr,
            password: config.walletPassword
        )

        try taker.setupLogging(dataDir: config.dataDir, logLevel: "Info")

        try taker.syncOfferbookAndWait()
        print("Offerbook synchronized")
        
        let offers = try taker.fetchOffers()
        print("Fetched offers: \(offers)")
        fflush(stdout)
        let _ = try taker.getWalletName()
        let balances = try taker.getBalances()
        XCTAssertEqual(balances.spendable, 0)

        let address = try taker.getNextExternalAddress(addressType: AddressType(addrType: "P2TR"))
        try fundAddress(address.address, config: config)
        try taker.syncAndSave()
        let updatedBalances = try taker.getBalances()
        XCTAssertGreaterThanOrEqual(updatedBalances.spendable, 0)

        if config.performSwap {
            let params = SwapParams(
                protocol: "Taproot",
                sendAmount: config.swapAmount,
                makerCount: 2,
                txCount: nil,
                requiredConfirms: nil,
                manuallySelectedOutpoints: nil,
                preferredMakers: nil
            )
            let swapId = try taker.prepareCoinswap(swapParams: params)
            let report = try taker.startCoinswap(swapId: swapId)
            let inputTotal = report.inputUtxos.reduce(Int64(0), +)
            let incomingTotal = Int64(report.incomingAmount)
            let changeTotal = report.outputChangeAmounts.reduce(Int64(0), +)
            let swapTotal = report.outputSwapAmounts.reduce(Int64(0), +)
            let makerFeeTotal = report.makerFeeInfo.reduce(0.0) { $0 + $1.totalFee }

            // Swap parameters
            XCTAssertEqual(report.outgoingAmount, Int64(config.swapAmount))
            XCTAssertEqual(Int64(report.makersCount ?? 0), 2)
            XCTAssertGreaterThan(inputTotal, 0)
            XCTAssertGreaterThan(incomingTotal, 0)

            // Fee information invariants
            XCTAssertEqual(inputTotal - incomingTotal, abs(report.feePaidOrEarned))
            XCTAssertEqual(report.totalMakerFees + report.miningFee, abs(report.feePaidOrEarned))
            assertApprox(makerFeeTotal, Double(report.totalMakerFees), tolerance: 2.0)

            // Output amount invariants
            XCTAssertGreaterThanOrEqual(report.outputChangeAmounts.count, 1)
            XCTAssertGreaterThanOrEqual(report.outputSwapAmounts.count, 1)
            XCTAssertEqual(changeTotal + swapTotal, incomingTotal)
            XCTAssertGreaterThan(swapTotal, 0)
            XCTAssertLessThanOrEqual(swapTotal, report.outgoingAmount)
        }
    }
}
