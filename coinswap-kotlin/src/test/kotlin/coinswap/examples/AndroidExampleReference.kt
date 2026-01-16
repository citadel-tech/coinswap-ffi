/**
 * Example: Android Integration Reference
 * 
 * This file demonstrates how to integrate coinswap into an downstream Android application.
 */

package coinswap.examples

/*
import android.os.Bundle
import android.util.Log
import androidx.appcompat.app.AppCompatActivity
import androidx.lifecycle.lifecycleScope
import uniffi.coinswap.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

class WalletActivity : AppCompatActivity() {
    private lateinit var taker: Taker
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Initialize in background thread
        lifecycleScope.launch(Dispatchers.IO) {
            try {
                taker = Taker.init(
                    dataDir = filesDir.absolutePath,
                    walletFileName = "wallet",
                    rpcConfig = getRpcConfig(),
                    controlPort = 9051u,
                    torAuthPassword = null,
                    zmqAddr = "tcp://localhost:28332",
                    password = getUserPassword()
                )
                
                taker.setupLogging(filesDir.absolutePath)
                
                // Wait for offerbook to sync
                Log.d("Wallet", "Waiting for offerbook sync...")
                while (taker.isOfferbookSyncing()) {
                    Log.d("Wallet", "Offerbook syncing...")
                    kotlinx.coroutines.delay(2000)
                }
                Log.d("Wallet", "Offerbook synchronized!")
                
                taker.syncAndSave()
                
                withContext(Dispatchers.Main) {
                    updateUI()
                }
            } catch (e: TakerError) {
                Log.e("Wallet", "Error: ${e.message}")
            }
        }
    }
    
    private fun performSwap(amount: Long) {
        lifecycleScope.launch(Dispatchers.IO) {
            try {
                val params = SwapParams(
                    sendAmount = amount.toULong(),
                    makerCount = 2u,
                    manuallySelectedOutpoints = null
                )
                
                val report = taker.doCoinswap(params)
                withContext(Dispatchers.Main) {
                    showSwapResult(report)
                }
            } catch (e: TakerError) {
                Log.e("Swap", "Swap failed: ${e.message}")
            }
        }
    }
    
    private fun getRpcConfig(): RPCConfig {
        return RPCConfig(
            url = "http://localhost:18442",
            user = "user",
            password = "password",
            walletName = "kotlin_test_taker"
        )
    }
    
    private fun getUserPassword(): String {
        // In production, retrieve from secure storage
        return "secure_password_123"
    }
    
    private fun updateUI() {
        // Update UI with wallet data
    }
    
    private fun showSwapResult(report: SwapReport?) {
        // Show swap results to user
        report?.let {
            Log.d("Swap", "Swap completed: ${it.amountSwapped} sats swapped")
        }
    }
}
*/

// Reference implementation only - see comments above
class AndroidExampleReference {
    fun info(): String {
        return """
            This is a reference implementation for Android integration.
            See the commented code in this file for the full example.
            
            Key points:
            1. Use lifecycleScope for coroutine management
            2. Run FFI operations on Dispatchers.IO
            3. Handle TakerError exceptions
            4. Sync wallet operations properly
            5. Use secure storage for passwords
        """.trimIndent()
    }
}
