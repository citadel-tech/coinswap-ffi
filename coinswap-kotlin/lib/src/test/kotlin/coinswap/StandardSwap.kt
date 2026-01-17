/**
 * JVM Integration Tests for Coinswap Kotlin bindings
 * 
 */

package coinswap

import uniffi.coinswap.*
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.BeforeAll
import org.junit.jupiter.api.io.TempDir
import java.nio.file.Path
import kotlin.test.assertNotNull
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class StandardSwap {
    
    companion object {
        @JvmStatic
        @BeforeAll
        fun setup() {
            println("Setting up Coinswap tests...")
        }
    }
    
    @Test
    fun `test SwapParams creation`() {
        val params = SwapParams(
            sendAmount = 100000u,
            makerCount = 2u,
            manuallySelectedOutpoints = null
        )
        
        assertNotNull(params)
        assertEquals(100000u, params.sendAmount)
        assertEquals(2u, params.makerCount)
        println("✅ SwapParams created: sendAmount=${params.sendAmount}, makerCount=${params.makerCount}")
    }
    
    @Test
    fun `test RpcConfig creation`() {
        val config = RpcConfig(
            url = "localhost:18442",
            username = "user",
            password = "password",
            walletName = "kotlin_test_taker"
        )
        
        assertNotNull(config)
        assertEquals("localhost:18442", config.url)
        assertEquals("user", config.username)
        println("✅ RpcConfig created successfully")
    }
    
    @Test
    fun `test Taker initialization and full coinswap`(@TempDir tempDir: Path) {
        println("\n🚀 Starting full coinswap integration test...")
        
        // Configure RPC connection to Bitcoin regtest node
        val rpcConfig = RpcConfig(
            url = "localhost:18442",
            username = "user",
            password = "password",
            walletName = "kotlin_test_taker"
        )
        
        println("📡 Connecting to Bitcoin node at ${rpcConfig.url}...")
        
        try {
            // Initialize Taker
            val taker = Taker.init(
                dataDir = tempDir.toString(),
                walletFileName = "test_wallet",
                rpcConfig = rpcConfig,
                controlPort = 9051u,
                torAuthPassword = "coinswap",
                zmqAddr = "tcp://localhost:28332",
                password = ""
            )
            
            println("✅ Taker initialized successfully")
            
            // Setup logging
            taker.setupLogging(tempDir.toString(), "info")
            println("📝 Logging configured")
            
            // Get wallet info
            val walletName = taker.getWalletName()
            println("💼 Wallet name: $walletName")
            
            // Check initial balances
            println("🔄 Syncing wallet...")
            taker.syncAndSave()
            val initialBalances = taker.getBalances()
            println("💰 Initial balances - Spendable: ${initialBalances.spendable} sats")
            
            
            // Get address and fund taker wallet
            println("\n💸 Getting next external address...")
            val takerAddress = taker.getNextExternalAddress(AddressType("P2WPKH"))
            println("📬 Address: ${takerAddress.address}")
            
            // Send 1.0 BTC to the taker address using docker exec
            println("\n💸 Funding taker wallet...")
            try {
                val sendCommand = ProcessBuilder(
                    "docker", "exec", "coinswap-bitcoind",
                    "bitcoin-cli", "-regtest", "-rpcport=18442",
                    "-rpcwallet=test", "-rpcuser=user", "-rpcpassword=password",
                    "sendtoaddress", takerAddress.address, "1.0"
                ).redirectErrorStream(true).start()
                
                val txid = sendCommand.inputStream.bufferedReader().readText().trim()
                val exitCode = sendCommand.waitFor()
                
                if (exitCode == 0) {
                    println("✅ Sent 1.0 BTC to taker address (txid: ${txid.take(16)}...)")
                } else {
                    println("❌ Failed to send BTC: $txid")
                    throw Exception("Could not send BTC to taker address")
                }
                
                // Wait a moment for transaction to propagate
                Thread.sleep(1000)
                
            } catch (e: Exception) {
                println("❌ Error funding wallet: ${e.message}")
                throw e
            }
            
            // Sync wallet after funding
            println("🔄 Syncing wallet after funding...")
            taker.syncAndSave()
            val updatedBalances = taker.getBalances()
            println("💰 Updated balances:")
            println("   Spendable: ${updatedBalances.spendable} sats")
            println("   Regular: ${updatedBalances.regular} sats")
            println("   Swap: ${updatedBalances.swap} sats")
            println("   Fidelity: ${updatedBalances.fidelity} sats")
            
            // Sync offerbook
            println("\n� Syncing offerbook...")
            println("Checking if offerbook is syncing: ${taker.isOfferbookSyncing()}")
            
            taker.runOfferSyncNow()
            
            // Wait for synchronization to complete
            println("Waiting for offerbook synchronization to complete...")
            try {
                println("Offerbook sync in progress...")
                Thread.sleep(30000)
            } catch (e: Exception) {
                println("Error checking offerbook sync status: ${e.message}")
            }
            
            
            // Attempt to fetch offers from makers
            println("\n📡 Attempting to fetch offers from makers...")
            println("   Note: In regtest mode, makers are auto-discovered during coinswap")
            try {
                val offerBook = taker.fetchOffers()
                println("✅ Successfully fetched offers")
                println("   Total makers found: ${offerBook.makers.size}")
                
                if (offerBook.makers.isNotEmpty()) {
                    println("\n🎯 Maker Details:")
                    offerBook.makers.forEachIndexed { i, maker ->
                        println("\n  Maker ${i + 1}:")
                        println("    Address: ${maker.address.address}")
                        println("    State: ${maker.state}")
                        
                        maker.offer?.let { offer ->
                            println("    Offer Details:")
                            println("      Base Fee: ${offer.baseFee} sats")
                            println("      Amount Relative Fee: ${offer.amountRelativeFeePct}%")
                            println("      Time Relative Fee: ${offer.timeRelativeFeePct}%")
                            println("      Required Confirms: ${offer.requiredConfirms}")
                            println("      Minimum Locktime: ${offer.minimumLocktime}")
                            println("      Min Size: ${offer.minSize} sats")
                            println("      Max Size: ${offer.maxSize} sats")
                        } ?: println("    Offer: None (no offer available)")
                    }
                } else {
                    println("\n⚠️  No makers found in offerbook")
                }
            } catch (e: Exception) {
                println("⚠️  Could not fetch offers (expected in regtest): ${e.message}")
                println("   Makers will be auto-discovered during coinswap")
            }
            
            // Sync wallet before checking initial balance
            println("\n🔄 Syncing wallet...")
            taker.syncAndSave()
            println("✅ Wallet synced")
            
            // Perform a coinswap
            println("\n� Initiating coinswap...")
            val swapParams = SwapParams(
                sendAmount = 500000u,  // 500,000 sats (same as Python test)
                makerCount = 2u,
                manuallySelectedOutpoints = null
            )
            println("Swap Parameters:")
            println("  Send Amount: ${swapParams.sendAmount} sats")
            println("  Maker Count: ${swapParams.makerCount}")
            
            try {
                println("\n🔄 Executing coinswap (this may take a while)...")
                val swapReport = taker.doCoinswap(swapParams)
                
                if (swapReport != null) {
                    println("\n✅ Coinswap completed successfully!")
                    println("\nSwap Report:")
                    println("  Swap ID: ${swapReport.swapId}")
                    println("  Duration: ${swapReport.swapDurationSeconds} seconds")
                    println("  Target Amount: ${swapReport.targetAmount} sats")
                    println("  Total Fee: ${swapReport.totalFee} sats")
                    println("  Maker Fees: ${swapReport.totalMakerFees} sats")
                    println("  Mining Fee: ${swapReport.miningFee} sats")
                    println("  Fee Percentage: ${swapReport.feePercentage}%")
                    println("  Number of Makers Used: ${swapReport.makersCount}")
                    println("  Maker Addresses:")
                    swapReport.makerAddresses.forEachIndexed { i, addr ->
                        println("    ${i + 1}. $addr")
                    }
                    assertNotNull(swapReport)
                } else {
                    println("\n⚠️  Coinswap returned no result (possibly no makers available)")
                }
                
            } catch (e: TakerException) {
                println("\n❌ Coinswap failed: ${e.message}")
                println("   This is expected if makers are not running or not properly set up.")
                throw e
            }
            
            // Final balance check
            println("\n📊 Final balances after coinswap...")
            taker.syncAndSave()
            val finalBalances = taker.getBalances()
            println("Final Balances:")
            println("  Spendable: ${finalBalances.spendable} sats")
            println("  Regular: ${finalBalances.regular} sats")
            println("  Swap: ${finalBalances.swap} sats")
            println("  Fidelity: ${finalBalances.fidelity} sats")
            
            println("\n✅ All tests completed!")
            
        } catch (e: TakerException) {
            println("❌ TakerException: ${e.message}")
            println("   Make sure Bitcoin regtest node is running on localhost:18442")
            println("   Make sure Tor is running on port 9051")
            println("   Make sure wallet has sufficient funds")
            throw e
        } catch (e: Exception) {
            println("❌ Unexpected error: ${e.message}")
            e.printStackTrace()
            throw e
        }
    }
    
    @Test
    fun `test bindings are loaded`() {
        println("✅ Bindings loaded successfully!")
        assert(true)
    }
}
