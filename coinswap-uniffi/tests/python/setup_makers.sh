#!/usr/bin/env bash
set -euo pipefail

# --- Config ---
COINSWAP_REPO="https://github.com/citadel-tech/coinswap.git"
COINSWAP_DIR="$HOME/coinswap"
MAKER1_DIR="$HOME/.coinswap/maker1"
MAKER2_DIR="$HOME/.coinswap/maker2"

MAKER1_RPC_PORT=6103
MAKER2_RPC_PORT=16103

RPC_USER=user
RPC_PASS=password

# --- Helpers ---
btc() {
    bitcoin-cli -regtest -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASS" "$@"
}

makercli() {
    "$COINSWAP_DIR/target/release/maker-cli" "$@"
}

# --- Preconditions ---
[ -d "$COINSWAP_DIR/.git" ] || git clone "$COINSWAP_REPO" "$COINSWAP_DIR"

[ -x "$COINSWAP_DIR/target/release/makerd" ] || \
    (cd "$COINSWAP_DIR" && cargo build --release)

btc getblockcount >/dev/null

btc createwallet test_funding_wallet false false "" false true 2>/dev/null || \
btc loadwallet test_funding_wallet >/dev/null

if [ "$(btc getblockcount)" -lt 101 ]; then
    addr=$(btc -rpcwallet=test_funding_wallet getnewaddress)
    btc generatetoaddress 101 "$addr" >/dev/null
fi

# --- Maker config ---
mkdir -p "$MAKER1_DIR" "$MAKER2_DIR"

cat > "$MAKER2_DIR/config.toml" <<EOF
network_port = 16102
rpc_port = 16103
socks_port = 9050
control_port = 9051
tor_auth_password = ""
min_swap_amount = 10000
fidelity_amount = 50000
fidelity_timelock = 13104
connection_type = "CLEARNET"
base_fee = 100
amount_relative_fee_pct = 0.1
EOF

# --- Start makers ---
cd "$COINSWAP_DIR"

start_maker() {
    local dir=$1 wallet=$2 log=$3

    nohup ./target/release/makerd \
        -r 127.0.0.1:18443 \
        -a "$RPC_USER:$RPC_PASS" \
        -z tcp://127.0.0.1:28332 \
        -d "$dir" \
        -w "$wallet" \
        > "$log" 2>&1 &

    echo $! > "$dir/makerd.pid"
}

start_maker "$MAKER1_DIR" maker1-wallet "$MAKER1_DIR/makerd.log"
start_maker "$MAKER2_DIR" maker2-wallet "$MAKER2_DIR/makerd.log"

sleep 10

# --- Fund makers ---
fund_maker() {
    local rpc_port=$1 dir=$2 log=$3

    for _ in {1..30}; do
        makercli -p "127.0.0.1:$rpc_port" send-ping &>/dev/null && break
        sleep 1
    done

    if grep -q "Waiting for funds" "$log"; then
        addr=$(grep "send coins to address" "$log" | tail -1 | awk '{print $NF}')
        btc -rpcwallet=test_funding_wallet sendtoaddress "$addr" 0.1 >/dev/null
        mine=$(btc -rpcwallet=test_funding_wallet getnewaddress)
        btc generatetoaddress 1 "$mine" >/dev/null
        makercli -p "127.0.0.1:$rpc_port" sync-wallet >/dev/null
    fi
}

fund_maker "$MAKER1_RPC_PORT" "$MAKER1_DIR" "$MAKER1_DIR/makerd.log"
fund_maker "$MAKER2_RPC_PORT" "$MAKER2_DIR" "$MAKER2_DIR/makerd.log"

# --- Wait for setup ---
sleep 30

# --- Verify ---
makercli -p "127.0.0.1:$MAKER1_RPC_PORT" send-ping >/dev/null
makercli -p "127.0.0.1:$MAKER2_RPC_PORT" send-ping >/dev/null

makercli -p "127.0.0.1:$MAKER1_RPC_PORT" get-balances | grep -i spendable || true
makercli -p "127.0.0.1:$MAKER2_RPC_PORT" get-balances | grep -i spendable || true