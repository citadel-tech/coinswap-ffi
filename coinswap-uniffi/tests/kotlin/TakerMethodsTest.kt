package tech.citadel.coinswap.tests

import org.junit.After
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test
// Import the UniFFI-generated bindings from coinswap.kt
import uniffi.coinswap.*
import java.io.File
import java.nio.file.Files
import java.nio.file.Paths

/**
 * FFI Layer Tests for Coinswap Taker (Kotlin)
 *
 * This test suite validates the UniFFI Kotlin bindings for the Coinswap Taker,
 * ensuring the FFI layer correctly wraps the underlying Rust API.
 *
 * Based on the Rust FFI test patterns.
 */
class TakerMethodsTest {

    private lateinit var bitcoind: Process
    private lateinit var bitcoindDir: File
    private var rpcPort: Int = 18443

    @Before
    fun setup() {
        // Setup test bitcoind instance
        bitcoindDir = Files.createTempDirectory("coinswap_kotlin_test").toFile()
        
        // Start bitcoind in regtest mode
        val bitcoindCmd = listOf(
            "bitcoind",
            "-regtest",
            "-datadir=${bitcoindDir.absolutePath}",
            "-rpcport=$rpcPort",
            "-rpcuser=test_user",
            "-rpcpassword=test_pass",
            "-txindex=1",
            "-fallbackfee=0.00001",
            "-daemon"
        )
        
        ProcessBuilder(bitcoindCmd)
            .redirectOutput(ProcessBuilder.Redirect.INHERIT)
            .redirectError(ProcessBuilder.Redirect.INHERIT)
            .start()
            .waitFor()
        
        // Give bitcoind time to start
        Thread.sleep(2000)
        
        // Generate initial blocks for coinbase maturity
        generateBlocks(101)
    }

    @After
    fun teardown() {
        // Stop bitcoind
        ProcessBuilder(
            "bitcoin-cli",
            "-regtest",
            "-datadir=${bitcoindDir.absolutePath}",
            "-rpcport=$rpcPort",
            "-rpcuser=test_user",
            "-rpcpassword=test_pass",
            "stop"
        ).start().waitFor()
        
        // Cleanup test directories
        cleanupWallet()
        bitcoindDir.deleteRecursively()
    }

    private fun generateBlocks(count: Int) {
        val address = getNewAddress()
        ProcessBuilder(
            "bitcoin-cli",
            "-regtest",
            "-datadir=${bitcoindDir.absolutePath}",
            "-rpcport=$rpcPort",
            "-rpcuser=test_user",
            "-rpcpassword=test_pass",
            "generatetoaddress",
            count.toString(),
            address
        ).start().waitFor()
    }

    private fun getNewAddress(): String {
        val process = ProcessBuilder(
            "bitcoin-cli",
            "-regtest",
            "-datadir=${bitcoindDir.absolutePath}",
            "-rpcport=$rpcPort",
            "-rpcuser=test_user",
            "-rpcpassword=test_pass",
            "getnewaddress"
        ).start()
        
        return process.inputStream.bufferedReader().readText().trim().removeSurrounding("\"")
    }

    private fun sendToAddress(address: String, amount: Double): String {
        val process = ProcessBuilder(
            "bitcoin-cli",
            "-regtest",
            "-datadir=${bitcoindDir.absolutePath}",
            "-rpcport=$rpcPort",
            "-rpcuser=test_user",
            "-rpcpassword=test_pass",
            "sendtoaddress",
            address,
            amount.toString()
        ).start()
        
        return process.inputStream.bufferedReader().readText().trim().removeSurrounding("\"")
    }

    private fun setupTaker(walletName: String): Taker {
        val rpcConfig = RpcConfig(
            url = "127.0.0.1:$rpcPort",
            username = "test_user",
            password = "test_pass",
            walletName = walletName
        )

        return Taker.init(
            dataDir = null,
            walletFileName = walletName,
            rpcConfig = rpcConfig,
            controlPort = null,
            torAuthPassword = null,
            zmqAddr = "tcp://127.0.0.1:28332",
            password = null
        )
    }

    private fun fundTakerWallet(taker: Taker, amount: Double) {
        val addressInfo = taker.getNextExternalAddress()
        val fundingAddress = addressInfo.address
        
        // Send funds to the taker wallet
        sendToAddress(fundingAddress, amount)
        
        // Mine a block to confirm
        generateBlocks(1)
        
        // Sync wallet
        taker.syncAndSave()
    }

    private fun cleanupWallet() {
        val homeDir = System.getProperty("user.home")
        val walletDir = Paths.get(homeDir, ".coinswap", "taker", "wallets").toFile()
        
        if (walletDir.exists()) {
            walletDir.deleteRecursively()
        }
    }

    @Test
    fun testTakerGetBalance() {
        val taker = setupTaker("balance-taker")
        
        val balances = taker.getBalances()
        
        assertEquals("Initial spendable balance should be zero", 0L, balances.spendable)
        assertEquals("Initial regular balance should be zero", 0L, balances.regular)
        assertEquals("Initial swap balance should be zero", 0L, balances.swap)
        assertEquals("Initial fidelity balance should be zero", 0L, balances.fidelity)
    }

    @Test
    fun testTakerAddressGeneration() {
        val taker = setupTaker("address-taker")
        
        // Test external address generation
        val address1 = taker.getNextExternalAddress()
        assertNotNull("First external address should not be null", address1)
        assertTrue("First address should not be empty", address1.address.isNotEmpty())
        
        val address2 = taker.getNextExternalAddress()
        assertNotNull("Second external address should not be null", address2)
        assertTrue("Second address should not be empty", address2.address.isNotEmpty())
        
        // Addresses should be different
        assertNotEquals(
            "Generated addresses should be unique",
            address1.address,
            address2.address
        )
        
        // Test internal address generation
        val internalAddresses = taker.getNextInternalAddresses(3u)
        assertEquals(
            "Should generate 4 internal addresses (3 + 1)",
            4,
            internalAddresses.size
        )
    }

    @Test
    fun testTakerWalletFunding() {
        val taker = setupTaker("funding-taker")
        
        // Fund the wallet with 1 BTC
        fundTakerWallet(taker, 1.0)
        
        // Check balance
        val balances = taker.getBalances()
        val expectedSats = 100_000_000L // 1 BTC in satoshis
        
        assertEquals(
            "Spendable balance should be 1 BTC",
            expectedSats,
            balances.spendable
        )
        
        println("✅ Wallet funded successfully: ${balances.spendable} sats")
    }

    @Test
    fun testTakerListUtxos() {
        val taker = setupTaker("utxo-taker")
        
        // Initially should have no UTXOs
        val initialUtxos = taker.listAllUtxoSpendInfo()
        assertEquals("Should start with no UTXOs", 0, initialUtxos.size)
        
        // Fund wallet twice with different amounts
        fundTakerWallet(taker, 0.5)
        fundTakerWallet(taker, 1.0)
        
        // Confirm transactions
        generateBlocks(1)
        taker.syncAndSave()
        
        // Check UTXOs
        val utxos = taker.listAllUtxoSpendInfo()
        assertEquals("Should have 2 UTXOs after funding", 2, utxos.size)
        
        println("✅ Found ${utxos.size} UTXOs")
    }

    @Test
    fun testSwapParamsCreation() {
        // Test SwapParams struct creation
        val swapParams = SwapParams(
            sendAmount = 500_000u, // 0.005 BTC in sats
            makerCount = 2u,
            manuallySelectedOutpoints = null
        )
        
        assertEquals(500_000u, swapParams.sendAmount)
        assertEquals(2u, swapParams.makerCount)
        assertNull(swapParams.manuallySelectedOutpoints)
        
        // Test with manual outpoints
        val swapParamsWithSelection = SwapParams(
            sendAmount = 1_000_000u,
            makerCount = 3u,
            manuallySelectedOutpoints = emptyList()
        )
        
        assertEquals(1_000_000u, swapParamsWithSelection.sendAmount)
        assertEquals(3u, swapParamsWithSelection.makerCount)
        assertNotNull(swapParamsWithSelection.manuallySelectedOutpoints)
    }

    @Test
    fun testTakerSync() {
        val taker = setupTaker("sync-taker")
        
        // Sync should succeed even with empty wallet
        taker.syncAndSave()
        
        println("✅ Sync successful")
    }

    @Test
    fun testRpcConfigConversion() {
        val rpcConfig = RpcConfig(
            url = "127.0.0.1:18443",
            username = "test_user",
            password = "test_pass",
            walletName = "test_wallet"
        )
        
        // Test that config values are preserved
        assertEquals("127.0.0.1:18443", rpcConfig.url)
        assertEquals("test_user", rpcConfig.username)
        assertEquals("test_pass", rpcConfig.password)
        assertEquals("test_wallet", rpcConfig.walletName)
    }

    @Test
    fun testTakerGetTransactions() {
        val taker = setupTaker("tx-taker")
        
        // Initially should have no transactions
        val initialTxs = taker.getTransactions(null, null)
        assertEquals("Should start with no transactions", 0, initialTxs.size)
        
        // Fund the wallet
        fundTakerWallet(taker, 0.1)
        
        // Should now have transactions
        val txs = taker.getTransactions(null, null)
        assertTrue(
            "Should have transactions after funding",
            txs.size > 0
        )
        
        println("Initial transactions: ${initialTxs.size}, Final transactions: ${txs.size}")
    }

    @Test
    fun testTakerWalletName() {
        val walletName = "named-wallet-test"
        val taker = setupTaker(walletName)
        
        val retrievedName = taker.getWalletName()
        assertEquals("Wallet name should match", walletName, retrievedName)
        
        println("✅ Wallet name: $retrievedName")
    }

    @Test
    fun testMultipleTakerInstances() {
        // Create first taker (multisig)
        val taker1 = setupTaker("multi-taker-1")
        
        // Create second taker (taproot)
        val rpcConfig2 = RpcConfig(
            url = "127.0.0.1:$rpcPort",
            username = "test_user",
            password = "test_pass",
            walletName = "multi-taker-2"
        )
        
        val taker2 = TaprootTaker.init(
            dataDir = null,
            walletFileName = "multi-taker-2",
            rpcConfig = rpcConfig2,
            controlPort = null,
            torAuthPassword = null,
            zmqAddr = "tcp://127.0.0.1:28332",
            password = null
        )
        
        // Fund both takers
        fundTakerWallet(taker1, 1.0)
        
        val address2Info = taker2.getNextExternalAddress()
        sendToAddress(address2Info.address, 2.5)
        generateBlocks(1)
        taker2.syncAndSave()
        
        // Check balances
        val balance1 = taker1.getBalances()
        val balance2 = taker2.getBalances()
        
        assertEquals(
            "First taker (multisig) should have 1 BTC",
            100_000_000L,
            balance1.spendable
        )
        assertEquals(
            "Second taker (taproot) should have 2.5 BTC",
            250_000_000L,
            balance2.spendable
        )
        
        println("✅ Multisig taker balance: ${balance1.spendable} sats")
        println("✅ Taproot taker balance: ${balance2.spendable} sats")
    }
}
