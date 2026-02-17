import Foundation
import XCTest
import Coinswap

final class LiveTaprootSwapTests: XCTestCase {
    func testLiveTaprootTakerFlow() throws {
        try requireLiveTestsEnabled()
        try cleanupCoinswapData(walletName: "swift_taproot_wallet")
        let config = try LiveTestConfig(walletNameOverride: "swift_taproot_wallet")

        let taker = try TaprootTaker.`init`(
            dataDir: config.dataDir,
            walletFileName: config.walletName,
            rpcConfig: config.rpcConfig,
            controlPort: config.torControlPort,
            torAuthPassword: config.torAuthPassword,
            zmqAddr: config.zmqAddr,
            password: config.walletPassword
        )

        try taker.setupLogging(dataDir: config.dataDir, logLevel: "Info")

        try taker.runOfferSyncNow()
        Thread.sleep(forTimeInterval: 30.0)
        print("Offerbook sync status: \(try taker.isOfferbookSyncing())")
        while !(try taker.isOfferbookSyncing()) {
            print("Offerbook not syncing yet, triggering sync...")
            // try taker.runOfferSyncNow()
            Thread.sleep(forTimeInterval: 5.0)
        }
        
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
            let params = TaprootSwapParams(
                sendAmount: config.swapAmount,
                makerCount: 2,
                txCount: nil,
                requiredConfirms: nil,
                manuallySelectedOutpoints: nil
            )
            let report = try taker.doCoinswap(swapParams: params)
            if let report = report {
                // Swap parameters
                XCTAssertEqual(report.targetAmount, 500000)
                XCTAssertEqual(report.totalInputAmount, 100000000)
                assertApprox(report.totalOutputAmount, 99995995)
                XCTAssertEqual(Int64(report.makersCount), 2)

                // Fee information
                assertApprox(report.totalFee, 4005)
                XCTAssertEqual(report.totalMakerFees, 2696)
                assertApprox(report.miningFee, 1309)

                // Maker 1 fee details
                XCTAssertEqual(report.makerFeeInfo.count, 2)
                XCTAssertEqual(report.makerFeeInfo[0].baseFee, 100.0, accuracy: 0.01)
                XCTAssertEqual(report.makerFeeInfo[0].amountRelativeFee, 500.0, accuracy: 0.01)
                XCTAssertEqual(report.makerFeeInfo[0].timeRelativeFee, 1000.0, accuracy: 0.01)
                XCTAssertEqual(report.makerFeeInfo[0].totalFee, 1600.0, accuracy: 0.01)

                // Maker 2 fee details
                XCTAssertEqual(report.makerFeeInfo[1].baseFee, 100.0, accuracy: 0.01)
                XCTAssertEqual(report.makerFeeInfo[1].amountRelativeFee, 498.40, accuracy: 0.01)
                XCTAssertEqual(report.makerFeeInfo[1].timeRelativeFee, 498.40, accuracy: 0.01)
                XCTAssertEqual(report.makerFeeInfo[1].totalFee, 1096.80, accuracy: 0.01)
         
                // Output change amounts
                XCTAssertEqual(report.outputChangeAmounts.count, 1)
                assertApprox(report.outputChangeAmounts[0], 99499694)

                // Output swap amounts
                XCTAssertEqual(report.outputSwapAmounts.count, 1)
                assertApprox(report.outputSwapAmounts[0], 496301)
            }
        }
    }
}
