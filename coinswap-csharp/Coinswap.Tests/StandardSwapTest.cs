using System.Diagnostics;
using Coinswap;
using Coinswap.Native;
using Xunit;
using Xunit.Abstractions;

namespace Coinswap.Tests;

/// <summary>
/// Live end-to-end 2-maker coinswap against the docker regtest stack, mirroring
/// coinswap-python/test/standard_swap.py. Requires the stack running:
/// <c>cd ../ffi-commons &amp;&amp; ./ffi-docker-setup start 4</c>.
/// </summary>
public class StandardSwapTest
{
    private const string WalletName = "csharp_legacy_wallet";
    private const string BitcoindContainer = "coinswap-bitcoind";
    private const string RpcUrl = "localhost:18442";
    private const string RpcUser = "user";
    private const string RpcPassword = "password";
    private const string ZmqAddr = "tcp://127.0.0.1:28332";
    private const ushort ControlPort = 9051;

    private readonly ITestOutputHelper _out;

    public StandardSwapTest(ITestOutputHelper output) => _out = output;

    private static string DataDir =>
        Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.UserProfile),
            ".coinswap", "taker");

    [Fact]
    public void StandardTaprootSwapCompletes()
    {
        CleanupTestWallets();

        var rpc = new RpcConfig(
            Url: RpcUrl,
            Username: RpcUser,
            Password: RpcPassword,
            WalletName: WalletName);

        _out.WriteLine("Initializing Taker...");
        using var taker = CoinswapClient.Init(
            zmqAddr: ZmqAddr,
            dataDir: DataDir,
            walletFileName: WalletName,
            rpcConfig: rpc,
            controlPort: ControlPort,
            torAuthPassword: "coinswap",
            password: "");

        try { taker.SetupLogging("Info", DataDir); }
        catch (Exception e) { _out.WriteLine($"warning: could not setup logging: {e.Message}"); }

        _out.WriteLine($"Wallet name: {taker.GetWalletName()}");

        _out.WriteLine("Syncing offerbook...");
        taker.SyncOfferBook();

        var offerbook = taker.FetchOffers();
        _out.WriteLine($"Makers found: {offerbook.Makers.Length}");
        Assert.True(offerbook.Makers.Length >= 2,
            $"need at least 2 makers for the swap, found {offerbook.Makers.Length}");

        _out.WriteLine("Syncing wallet...");
        taker.SyncWallet();
        _out.WriteLine($"Initial balances: {taker.GetBalances()}");

        var address = taker.GetNextExternalAddress("P2WPKH");
        _out.WriteLine($"Funding address: {address}");
        FundTaker(address, "1.0");

        taker.SyncWallet();
        var funded = taker.GetBalances();
        _out.WriteLine($"Funded balances: spendable={funded.Spendable} regular={funded.Regular}");
        Assert.True(funded.Spendable > 0, "taker should have spendable balance after funding");

        _out.WriteLine("Executing coinswap...");
        var swapParams = new SwapParams(
            Protocol: "Taproot",
            SendAmount: 500_000,
            MakerCount: 2,
            TxCount: 1,
            RequiredConfirms: 1,
            ManuallySelectedOutpoints: null,
            PreferredMakers: null);

        var swapId = taker.PrepareCoinswap(swapParams);
        var report = taker.StartCoinswap(swapId);

        Assert.NotNull(report);
        _out.WriteLine($"Swap complete: id={report.SwapId} status={report.Status} " +
                       $"duration={report.SwapDurationSeconds:F2}s " +
                       $"outgoing={report.OutgoingAmount} feePaid={report.FeePaid}");

        taker.SyncWallet();
        _out.WriteLine($"Final balances: {taker.GetBalances()}");
    }

    /// <summary>Sends BTC to the taker address from the docker-hosted "test" wallet.</summary>
    private void FundTaker(string address, string amountBtc)
    {
        var (code, stdout, stderr) = RunDocker(
            "exec", BitcoindContainer, "bitcoin-cli", "-regtest", "-rpcport=18442",
            "-rpcwallet=test", $"-rpcuser={RpcUser}", $"-rpcpassword={RpcPassword}",
            "sendtoaddress", address, amountBtc);
        if (code != 0)
            throw new InvalidOperationException($"funding failed: {stderr}");
        _out.WriteLine($"Sent {amountBtc} BTC (txid: {stdout.Trim()[..Math.Min(16, stdout.Trim().Length)]}...)");
        Thread.Sleep(1000);
    }

    private void CleanupTestWallets()
    {
        var walletsDir = Path.Combine(DataDir, "wallets");
        if (Directory.Exists(walletsDir))
        {
            foreach (var path in Directory.GetFileSystemEntries(walletsDir))
            {
                if (!Path.GetFileName(path).StartsWith(WalletName)) continue;
                try
                {
                    if (Directory.Exists(path)) Directory.Delete(path, recursive: true);
                    else File.Delete(path);
                    _out.WriteLine($"cleaned {path}");
                }
                catch (Exception e) { _out.WriteLine($"warning: could not clean {path}: {e.Message}"); }
            }
        }
        // Best-effort: unload any stale bitcoind wallet of the same name.
        RunDocker("exec", BitcoindContainer, "bitcoin-cli", "-regtest", "-rpcport=18442",
            $"-rpcuser={RpcUser}", $"-rpcpassword={RpcPassword}", "unloadwallet", WalletName);
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
