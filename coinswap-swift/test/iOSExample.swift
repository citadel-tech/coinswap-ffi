/**
 * iOS Example for Coinswap Swift bindings
 * 
 * Demonstrates how to integrate coinswap into an iOS application
 * using UIKit and background threading.
 */

import UIKit
import Combine

class WalletViewController: UIViewController {
    private var taker: Taker?
    private var cancellables = Set<AnyCancellable>()
    
    override func viewDidLoad() {
        super.viewDidLoad()
        
        // Initialize in background
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            do {
                let documentsPath = FileManager.default.urls(
                    for: .documentDirectory, 
                    in: .userDomainMask
                )[0].path
                
                self?.taker = try Taker(
                    dataDir: documentsPath,
                    walletFileName: "wallet",
                    rpcConfig: self?.getRpcConfig(),
                    controlPort: 9051,
                    torAuthPassword: nil,
                    zmqAddr: "tcp://localhost:28332",
                    password: self?.getUserPassword()
                )
                
                try self?.taker?.setupLogging(dataDir: documentsPath)
                
                // Wait for offerbook to sync
                print("Waiting for offerbook synchronization...")
                try self?.taker?.syncOfferbookAndWait()
                print("Offerbook synchronized!")
                
                try self?.taker?.syncAndSave()
                
                DispatchQueue.main.async {
                    self?.updateUI()
                }
            } catch {
                fatalError("Initialization failed: \(error)")
            }
        }
    }
    
    func performSwap(amount: UInt64) {
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            do {
                let params = SwapParams(
                    sendAmount: amount,
                    makerCount: 2,
                    manuallySelectedOutpoints: nil
                )
                
                let report = try self?.taker?.doCoinswap(swapParams: params)
                guard let report else {
                    fatalError("Swap completed without a swap report")
                }
                
                DispatchQueue.main.async {
                    self?.showSwapResult(report)
                }
            } catch {
                fatalError("Swap failed: \(error)")
            }
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
    
    private func updateUI() {
        // Update UI with wallet data
    }
    
    private func showSwapResult(_ report: SwapReport?) {
        // Show swap results to user
        if let report = report {
            print("Swap completed: \(report.targetAmount) sats swapped")
        }
    }
}
