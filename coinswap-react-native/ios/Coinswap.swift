import Foundation
import React
import Coinswap

@objc(Coinswap)
class CoinswapModule: NSObject {
  private var takers: [String: Taker] = [:]
  private let lock = NSLock()

  @objc
  static func requiresMainQueueSetup() -> Bool {
    false
  }

  @objc(setupLogging:level:resolve:reject:)
  func setupLogging(
    _ dataDir: String?,
    level: String,
    resolve: @escaping RCTPromiseResolveBlock,
    reject: @escaping RCTPromiseRejectBlock
  ) {
    do {
      try Taker.setupLogging(dataDir: dataDir, logLevel: level)
      resolve(nil)
    } catch {
      reject("COINSWAP_ERROR", error.localizedDescription, error)
    }
  }

  @objc(createTaker:resolve:reject:)
  func createTaker(
    _ config: NSDictionary,
    resolve: @escaping RCTPromiseResolveBlock,
    reject: @escaping RCTPromiseRejectBlock
  ) {
    do {
      let rpcConfigMap = config["rpcConfig"] as? [String: Any]
      let rpcConfig: RpcConfig?
      if let rpcConfigMap {
        rpcConfig = RpcConfig(
          url: rpcConfigMap["url"] as? String ?? "",
          username: rpcConfigMap["username"] as? String ?? "",
          password: rpcConfigMap["password"] as? String ?? "",
          walletName: rpcConfigMap["walletName"] as? String ?? ""
        )
      } else {
        rpcConfig = nil
      }

      let controlPortValue = config["controlPort"] as? NSNumber
      let taker = try Taker.`init`(
        dataDir: config["dataDir"] as? String,
        walletFileName: config["walletFileName"] as? String,
        rpcConfig: rpcConfig,
        controlPort: controlPortValue?.uint16Value,
        torAuthPassword: config["torAuthPassword"] as? String,
        zmqAddr: config["zmqAddr"] as? String ?? "",
        password: config["password"] as? String
      )

      let takerId = UUID().uuidString
      lock.lock()
      takers[takerId] = taker
      lock.unlock()

      resolve(takerId)
    } catch {
      reject("COINSWAP_ERROR", error.localizedDescription, error)
    }
  }

  @objc(disposeTaker:resolve:reject:)
  func disposeTaker(
    _ takerId: String,
    resolve: @escaping RCTPromiseResolveBlock,
    reject: @escaping RCTPromiseRejectBlock
  ) {
    lock.lock()
    takers.removeValue(forKey: takerId)
    lock.unlock()
    resolve(nil)
  }

  @objc(syncOfferbookAndWait:resolve:reject:)
  func syncOfferbookAndWait(
    _ takerId: String,
    resolve: @escaping RCTPromiseResolveBlock,
    reject: @escaping RCTPromiseRejectBlock
  ) {
    do {
      try requireTaker(takerId).syncOfferbookAndWait()
      resolve(nil)
    } catch {
      reject("COINSWAP_ERROR", error.localizedDescription, error)
    }
  }

  @objc(syncAndSave:resolve:reject:)
  func syncAndSave(
    _ takerId: String,
    resolve: @escaping RCTPromiseResolveBlock,
    reject: @escaping RCTPromiseRejectBlock
  ) {
    do {
      try requireTaker(takerId).syncAndSave()
      resolve(nil)
    } catch {
      reject("COINSWAP_ERROR", error.localizedDescription, error)
    }
  }

  @objc(getBalances:resolve:reject:)
  func getBalances(
    _ takerId: String,
    resolve: @escaping RCTPromiseResolveBlock,
    reject: @escaping RCTPromiseRejectBlock
  ) {
    do {
      let balances = try requireTaker(takerId).getBalances()
      resolve([
        "regular": balances.regular,
        "swap": balances.swap,
        "contract": balances.contract,
        "fidelity": balances.fidelity,
        "spendable": balances.spendable,
      ])
    } catch {
      reject("COINSWAP_ERROR", error.localizedDescription, error)
    }
  }

  @objc(getNextExternalAddress:addressType:resolve:reject:)
  func getNextExternalAddress(
    _ takerId: String,
    addressType: String,
    resolve: @escaping RCTPromiseResolveBlock,
    reject: @escaping RCTPromiseRejectBlock
  ) {
    do {
      let address = try requireTaker(takerId).getNextExternalAddress(addressType: AddressType(addrType: addressType))
      resolve(["address": address.address])
    } catch {
      reject("COINSWAP_ERROR", error.localizedDescription, error)
    }
  }

  @objc(prepareCoinswap:swapParams:resolve:reject:)
  func prepareCoinswap(
    _ takerId: String,
    swapParams: NSDictionary,
    resolve: @escaping RCTPromiseResolveBlock,
    reject: @escaping RCTPromiseRejectBlock
  ) {
    do {
      let sendAmount = (swapParams["sendAmount"] as? NSNumber)?.uint64Value ?? 0
      let makerCount = (swapParams["makerCount"] as? NSNumber)?.uint32Value ?? 0
      let txCount = (swapParams["txCount"] as? NSNumber)?.uint32Value
      let requiredConfirms = (swapParams["requiredConfirms"] as? NSNumber)?.uint32Value
      let preferredMakers = swapParams["preferredMakers"] as? [String]

      let params = SwapParams(
        protocol: swapParams["protocol"] as? String,
        sendAmount: sendAmount,
        makerCount: makerCount,
        txCount: txCount,
        requiredConfirms: requiredConfirms,
        manuallySelectedOutpoints: nil,
        preferredMakers: preferredMakers
      )

      let swapId = try requireTaker(takerId).prepareCoinswap(swapParams: params)
      resolve(swapId)
    } catch {
      reject("COINSWAP_ERROR", error.localizedDescription, error)
    }
  }

  @objc(startCoinswap:swapId:resolve:reject:)
  func startCoinswap(
    _ takerId: String,
    swapId: String,
    resolve: @escaping RCTPromiseResolveBlock,
    reject: @escaping RCTPromiseRejectBlock
  ) {
    do {
      let report = try requireTaker(takerId).startCoinswap(swapId: swapId)
      resolve([
        "swapId": report.swapId,
        "outgoingAmount": report.outgoingAmount,
        "incomingAmount": report.incomingAmount,
        "feePaidOrEarned": report.feePaidOrEarned,
        "makersCount": report.makersCount ?? 0,
        "makerAddresses": report.makerAddresses,
        "totalMakerFees": report.totalMakerFees,
        "miningFee": report.miningFee,
        "status": report.status,
      ])
    } catch {
      reject("COINSWAP_ERROR", error.localizedDescription, error)
    }
  }

  private func requireTaker(_ takerId: String) throws -> Taker {
    lock.lock()
    defer { lock.unlock() }

    guard let taker = takers[takerId] else {
      throw NSError(domain: "coinswap-react-native", code: 1, userInfo: [NSLocalizedDescriptionKey: "Unknown taker id: \(takerId)"])
    }

    return taker
  }
}
