using System.Diagnostics;
using Coinswap.Native;
using Xunit;
using Xunit.Abstractions;

namespace Coinswap.Tests;

/// <summary>
/// Live integration test: 4 takers × 2 makers.
///
/// One test per (backend × protocol) combination — legacy/taproot over
/// rpc/electrum — each running a 2-maker coinswap against the Docker regtest
/// stack (1 RPC maker + 1 Electrum maker). Mirrors the Rust <c>swap_test</c> and
/// the Kotlin <c>SwapTest</c>. Requires the stack running:
/// <c>cd ../ffi-commons &amp;&amp; ./ffi-docker-setup start</c>.
/// </summary>
public class SwapTest
{
    private const string BitcoindContainer = "coinswap-bitcoind";
    private const string RpcUrl = "localhost:18442";
    private const string RpcUser = "user";
    private const string RpcPassword = "password";
    private const string ZmqAddr = "tcp://localhost:28332";
    private const string ElectrumUrl = "tcp://localhost:50001";
    private const ushort ControlPort = 9051;

    /// <summary>Amount swapped by each taker, in sats. The taker is funded with 2×.</summary>
    private const ulong SwapAmount = 500_000;

    private enum Backend { Rpc, Electrum }

    private readonly ITestOutputHelper _out;

    public SwapTest(ITestOutputHelper output) => _out = output;

    [Fact]
    public void LegacyRpc() =>
        RunSwap("legacy_rpc", Backend.Rpc, "Legacy", "P2WPKH");

    [Fact]
    public void TaprootRpc() =>
        RunSwap("taproot_rpc", Backend.Rpc, "Taproot", "P2TR");

    [Fact]
    public void LegacyElectrum() =>
        RunSwap("legacy_electrum", Backend.Electrum, "Legacy", "P2WPKH");

    [Fact]
    public void TaprootElectrum() =>
        RunSwap("taproot_electrum", Backend.Electrum, "Taproot", "P2TR");

    /// <summary>Run one taker end-to-end: init → fund → sync → 2-maker coinswap → assert.</summary>
    private void RunSwap(string name, Backend backend, string protocol, string addrType)
    {
        _out.WriteLine($"\n=== {name} ({protocol}) ===");

        var dataDir = Path.Combine(Path.GetTempPath(), $"coinswap-csharp-{name}-{Guid.NewGuid():N}");
        Directory.CreateDirectory(dataDir);

        var rpcConfig = backend == Backend.Rpc
            ? new RpcConfig(
                Url: RpcUrl,
                Username: RpcUser,
                Password: RpcPassword,
                WalletName: $"csharp_{name}")
            : null;

        var backendConfig = backend == Backend.Electrum
            ? new BackendConfig(Kind: "electrum", Url: ElectrumUrl)
            : null;

        // Positional args mirror the Rust `Taker::init` signature order:
        // (data_dir, wallet_file_name, rpc_config, control_port, tor_auth_password,
        //  zmq_addr, password, nostr_relays, backend_config).
        using var taker = Taker.Init(
            dataDir,
            name,
            rpcConfig,
            ControlPort,
            "coinswap",
            ZmqAddr,
            "",
            null,
            backendConfig);

        taker.SyncOfferbookAndWait();

        // Fund with 2× the swap amount across 4 fresh external addresses.
        const string quarterBtc = "0.0025"; // 250,000 sats; 4× = 1,000,000 = 2 × SwapAmount
        for (var i = 0; i < 4; i++)
        {
            var addr = taker.GetNextExternalAddress(new AddressType(addrType)).Addr;
            Fund(addr, quarterBtc);
        }

        var funded = WaitForSpendable(taker, SwapAmount * 2);
        Assert.True(
            (ulong)funded.Spendable >= SwapAmount * 2,
            $"{name}: spendable ({funded.Spendable}) should reach funded amount ({SwapAmount * 2})");

        var swapId = taker.PrepareCoinswap(new SwapParams(
            Protocol: protocol,
            SendAmount: SwapAmount,
            MakerCount: 2,
            TxCount: 1,
            RequiredConfirms: 1,
            ManuallySelectedOutpoints: null,
            PreferredMakers: null));

        var report = taker.StartCoinswap(swapId);
        Assert.NotNull(report);
        Assert.Equal(2u, report.MakersCount);
        Assert.Contains("SUCCESS", report.Status.ToUpperInvariant());

        _out.WriteLine($"✓ {name} passed (swap_id {report.SwapId})");
    }

    /// <summary>Sync until spendable reaches <paramref name="target"/>, tolerating Electrum indexing lag.</summary>
    private Balances WaitForSpendable(Taker taker, ulong target)
    {
        for (var i = 0; i < 30; i++)
        {
            taker.SyncAndSave();
            var b = taker.GetBalances();
            if ((ulong)b.Spendable >= target) return b;
            Thread.Sleep(3000);
        }
        return taker.GetBalances();
    }

    /// <summary>Sends BTC to the taker address from the docker-hosted "test" wallet.</summary>
    private void Fund(string address, string amountBtc)
    {
        var (code, _, stderr) = RunDocker(
            "exec", BitcoindContainer, "bitcoin-cli", "-regtest", "-rpcport=18442",
            "-rpcwallet=test", $"-rpcuser={RpcUser}", $"-rpcpassword={RpcPassword}",
            "sendtoaddress", address, amountBtc);
        if (code != 0)
            throw new InvalidOperationException($"funding failed: {stderr}");
    }

    private static (int code, string stdout, string stderr) RunDocker(params string[] args)
    {
        var psi = new ProcessStartInfo("docker")
        {
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            UseShellExecute = false,
        };
        foreach (var a in args) psi.ArgumentList.Add(a);

        using var p = Process.Start(psi)!;
        var stdout = p.StandardOutput.ReadToEnd();
        var stderr = p.StandardError.ReadToEnd();
        p.WaitForExit();
        return (p.ExitCode, stdout, stderr);
    }
}
