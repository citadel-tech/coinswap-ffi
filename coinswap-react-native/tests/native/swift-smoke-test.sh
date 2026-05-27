#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
TMP_DIR=$(mktemp -d /private/tmp/coinswap-rn-swift-smoke.XXXXXX)
MODULE_CACHE=/private/tmp/coinswap-rn-swift-module-cache
trap 'rm -rf "$TMP_DIR"' EXIT
mkdir -p "$MODULE_CACHE"

cat > "$TMP_DIR/BindingsSmoke.swift" <<'SWIFT'
import Foundation

func assertGeneratedBindingShapes() {
  let addressType = AddressType(addrType: "P2TR")
  precondition(addressType.addrType == "P2TR")

  let rpc = RpcConfig(
    url: "localhost:18442",
    username: "user",
    password: "password",
    walletName: "wallet"
  )
  precondition(rpc.walletName == "wallet")

  let params = SwapParams(
    protocol: "Legacy",
    sendAmount: 500_000,
    makerCount: 2,
    txCount: 1,
    requiredConfirms: 1,
    manuallySelectedOutpoints: nil,
    preferredMakers: nil
  )
  precondition(params.sendAmount == 500_000)
}
SWIFT

swiftc -typecheck \
  "$ROOT_DIR/ios/generated/Coinswap.swift" \
  "$TMP_DIR/BindingsSmoke.swift" \
  -I "$ROOT_DIR/ios/generated" \
  -module-cache-path "$MODULE_CACHE"

echo "Swift smoke test passed: generated bindings typecheck and expose expected record APIs."
