/**
 * JVM integration test: 4 takers × 2 makers.
 *
 * Mirrors the Rust `swap_test`: one test per (backend × protocol) combination —
 * legacy/taproot over rpc/electrum — each running a 2-maker coinswap against the
 * Docker regtest stack (1 RPC maker + 1 Electrum maker).
 */

package org.coinswap

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import java.nio.file.Path
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class SwapTest {

    private enum class Backend { RPC, ELECTRUM }

    private val swapAmount = 500_000uL

    /** Fund [address] with [btc] BTC from the Docker bitcoind `test` wallet. */
    private fun fund(address: String, btc: String) {
        val p = ProcessBuilder(
            "docker", "exec", "coinswap-bitcoind",
            "bitcoin-cli", "-regtest", "-rpcport=18442",
            "-rpcwallet=test", "-rpcuser=user", "-rpcpassword=password",
            "sendtoaddress", address, btc,
        ).redirectErrorStream(true).start()
        val out = p.inputStream.bufferedReader().readText().trim()
        check(p.waitFor() == 0) { "funding failed: $out" }
    }

    /** Sync until spendable reaches [target], tolerating Electrum indexing lag. */
    private fun waitForSpendable(taker: Taker, target: ULong): Balances {
        repeat(30) {
            taker.syncAndSave()
            val b = taker.getBalances()
            if (b.spendable.toULong() >= target) return b
            Thread.sleep(3000)
        }
        return taker.getBalances()
    }

    /** Run one taker end-to-end: init → fund → sync → 2-maker coinswap → assert. */
    private fun runSwap(
        name: String,
        dataDir: Path,
        backend: Backend,
        protocol: String,
        addrType: String,
    ) {
        println("\n=== $name ($protocol) ===")

        val rpcConfig = if (backend == Backend.RPC) {
            RpcConfig("localhost:18442", "user", "password", "kotlin_$name")
        } else null
        val backendConfig = if (backend == Backend.ELECTRUM) {
            BackendConfig(kind = "electrum", url = "tcp://localhost:50001")
        } else null

        val taker = Taker.init(
            dataDir = dataDir.toString(),
            walletFileName = name,
            rpcConfig = rpcConfig,
            controlPort = 9051u,
            torAuthPassword = "coinswap",
            zmqAddr = "tcp://localhost:28332",
            password = "",
            nostrRelays = null,
            backendConfig = backendConfig,
        )

        taker.syncOfferbookAndWait()

        // Fund with 2x the swap amount across 4 fresh addresses.
        val quarterBtc = "0.0025" // 250,000 sats; 4x = 1,000,000 = 2 * swapAmount
        repeat(4) {
            val addr = taker.getNextExternalAddress(AddressType(addrType)).addr
            fund(addr, quarterBtc)
        }
        val funded = waitForSpendable(taker, swapAmount * 2uL)
        assertEquals(
            (swapAmount * 2uL).toLong(), funded.spendable,
            "$name: spendable should equal funded amount",
        )

        val swapId = taker.prepareCoinswap(
            SwapParams(
                protocol = protocol,
                sendAmount = swapAmount,
                makerCount = 2u,
                txCount = 1u,
                requiredConfirms = 1u,
                manuallySelectedOutpoints = null,
                preferredMakers = null,
            ),
        )
        val report = taker.startCoinswap(swapId)
        assertNotNull(report)
        assertEquals(2u, report.makersCount, "$name: should route through 2 makers")
        assertTrue(
            report.status.uppercase().contains("SUCCESS"),
            "$name: swap status was ${report.status}",
        )
        println("✓ $name passed (swap_id ${report.swapId})")
    }

    @Test
    fun `legacy rpc swap`(@TempDir dir: Path) =
        runSwap("legacy_rpc", dir, Backend.RPC, "Legacy", "P2WPKH")

    @Test
    fun `taproot rpc swap`(@TempDir dir: Path) =
        runSwap("taproot_rpc", dir, Backend.RPC, "Taproot", "P2TR")

    @Test
    fun `legacy electrum swap`(@TempDir dir: Path) =
        runSwap("legacy_electrum", dir, Backend.ELECTRUM, "Legacy", "P2WPKH")

    @Test
    fun `taproot electrum swap`(@TempDir dir: Path) =
        runSwap("taproot_electrum", dir, Backend.ELECTRUM, "Taproot", "P2TR")
}
