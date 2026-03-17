/**
 * SwiftUI Example for Coinswap Swift bindings
 * 
 * Demonstrates how to integrate coinswap into a SwiftUI application
 * using modern Swift concurrency (async/await).
 */

import SwiftUI

class WalletViewModel: ObservableObject {
    @Published var balance: UInt64 = 0
    @Published var isLoading = false
    @Published var errorMessage: String?
    
    private var taker: Taker?
    
    func initialize() async {
        isLoading = true
        do {
            let documentsPath = FileManager.default.urls(
                for: .documentDirectory,
                in: .userDomainMask
            )[0].path
            
            taker = try Taker(
                dataDir: documentsPath,
                walletFileName: "wallet",
                rpcConfig: getRpcConfig(),
                controlPort: 9051,
                torAuthPassword: nil,
                zmqAddr: "tcp://localhost:28332",
                password: getUserPassword()
            )
            
            try taker?.setupLogging(dataDir: documentsPath)
            
            // Wait for offerbook to sync
            print("Waiting for offerbook synchronization...")
            try taker?.syncOfferbookAndWait()
            print("Offerbook synchronized!")
            
            try taker?.syncAndSave()
            
            let balances = try taker?.getBalances()
            await MainActor.run {
                self.balance = balances?.total ?? 0
                self.isLoading = false
            }
        } catch {
            fatalError("Initialization failed: \(error)")
        }
    }
    
    func performSwap(amount: UInt64) async {
        isLoading = true
        do {
            let params = SwapParams(
                sendAmount: amount,
                makerCount: 2,
                manuallySelectedOutpoints: nil
            )
            
            let report = try taker?.doCoinswap(swapParams: params)
            guard let report else {
                fatalError("Swap completed without a swap report")
            }
            await MainActor.run {
                self.isLoading = false
                // Handle report
                print("Swap completed: \(report.targetAmount) sats")
            }
        } catch {
            fatalError("Swap failed: \(error)")
        }
    }
    
    private func getRpcConfig() -> RPCConfig {
        RPCConfig(
            url: "http://localhost:18442",
            user: "bitcoin",
            password: "bitcoin",
            walletName: "taker_wallet"
        )
    }
    
    private func getUserPassword() -> String {
        // In production, retrieve from Keychain
        return "secure_password_123"
    }
}

struct WalletView: View {
    @StateObject private var viewModel = WalletViewModel()
    
    var body: some View {
        VStack {
            if viewModel.isLoading {
                ProgressView()
            } else {
                Text("Balance: \(viewModel.balance) sats")
                    .font(.headline)
                
                Button("Perform Swap") {
                    Task {
                        await viewModel.performSwap(amount: 100_000)
                    }
                }
                .buttonStyle(.borderedProminent)
            }
            
            if let error = viewModel.errorMessage {
                Text("Error: \(error)")
                    .foregroundColor(.red)
            }
        }
        .task {
            await viewModel.initialize()
        }
    }
}
