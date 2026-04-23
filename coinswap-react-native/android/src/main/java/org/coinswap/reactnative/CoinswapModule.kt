package org.coinswap.reactnative

import com.facebook.react.bridge.Arguments
import com.facebook.react.bridge.Promise
import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.bridge.ReadableArray
import com.facebook.react.bridge.ReadableMap
import com.facebook.react.bridge.ReadableType
import com.facebook.react.bridge.WritableArray
import com.facebook.react.bridge.WritableMap
import com.facebook.react.module.annotations.ReactModule
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch
import org.coinswap.AddressType
import org.coinswap.RpcConfig
import org.coinswap.SwapParams
import org.coinswap.Taker
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap

@ReactModule(name = CoinswapModule.NAME)
class CoinswapModule(reactContext: ReactApplicationContext) : NativeCoinswapSpec(reactContext) {
  companion object {
    const val NAME = "Coinswap"
  }

  private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
  private val takers = ConcurrentHashMap<String, Taker>()

  override fun getName(): String = NAME

  override fun invalidate() {
    super.invalidate()
    scope.cancel()
    takers.clear()
  }

  override fun setupLogging(dataDir: String?, level: String, promise: Promise) {
    launchPromise(promise) {
      Taker.setupLogging(dataDir, level)
      null
    }
  }

  override fun createTaker(config: ReadableMap, promise: Promise) {
    launchPromise(promise) {
      val rpcConfig = config.getNullableMap("rpcConfig")?.let { rpc ->
        RpcConfig(
          url = rpc.getRequiredString("url"),
          username = rpc.getRequiredString("username"),
          password = rpc.getRequiredString("password"),
          walletName = rpc.getRequiredString("walletName"),
        )
      }

      val taker = Taker.init(
        config.getNullableString("dataDir"),
        config.getNullableString("walletFileName"),
        rpcConfig,
        config.getNullableInt("controlPort")?.toUInt(),
        config.getNullableString("torAuthPassword"),
        config.getRequiredString("zmqAddr"),
        config.getNullableString("password"),
      )

      val takerId = UUID.randomUUID().toString()
      takers[takerId] = taker
      takerId
    }
  }

  override fun disposeTaker(takerId: String, promise: Promise) {
    launchPromise(promise) {
      takers.remove(takerId)
      null
    }
  }

  override fun syncOfferbookAndWait(takerId: String, promise: Promise) {
    launchPromise(promise) {
      requireTaker(takerId).syncOfferbookAndWait()
      null
    }
  }

  override fun syncAndSave(takerId: String, promise: Promise) {
    launchPromise(promise) {
      requireTaker(takerId).syncAndSave()
      null
    }
  }

  override fun getBalances(takerId: String, promise: Promise) {
    launchPromise(promise) {
      val balances = requireTaker(takerId).getBalances()
      toBalancesMap(balances)
    }
  }

  override fun getNextExternalAddress(takerId: String, addressType: String, promise: Promise) {
    launchPromise(promise) {
      val address = requireTaker(takerId).getNextExternalAddress(AddressType(addressType))
      val result = Arguments.createMap()
      result.putString("address", readAsString(address, "address"))
      result
    }
  }

  override fun prepareCoinswap(takerId: String, swapParams: ReadableMap, promise: Promise) {
    launchPromise(promise) {
      val params = SwapParams(
        protocol = swapParams.getNullableString("protocol"),
        sendAmount = swapParams.getRequiredLong("sendAmount").toULong(),
        makerCount = swapParams.getRequiredInt("makerCount").toUInt(),
        txCount = swapParams.getNullableInt("txCount")?.toUInt(),
        requiredConfirms = swapParams.getNullableInt("requiredConfirms")?.toUInt(),
        manuallySelectedOutpoints = null,
        preferredMakers = swapParams.getNullableStringArray("preferredMakers"),
      )

      requireTaker(takerId).prepareCoinswap(params)
    }
  }

  override fun startCoinswap(takerId: String, swapId: String, promise: Promise) {
    launchPromise(promise) {
      val report = requireTaker(takerId).startCoinswap(swapId)
      toSwapReportMap(report)
    }
  }

  private fun requireTaker(takerId: String): Taker {
    return takers[takerId] ?: throw IllegalArgumentException("Unknown taker id: $takerId")
  }

  private fun launchPromise(promise: Promise, block: () -> Any?) {
    scope.launch {
      try {
        promise.resolve(block())
      } catch (error: Throwable) {
        promise.reject("COINSWAP_ERROR", error.message, error)
      }
    }
  }

  private fun readAsString(target: Any, vararg names: String): String {
    val value = readProperty(target, *names) ?: throw IllegalStateException("Missing string property")
    return value.toString()
  }

  private fun readAsDouble(target: Any, vararg names: String): Double {
    val value = readProperty(target, *names)
    return when (value) {
      null -> 0.0
      is Number -> value.toDouble()
      else -> value.toString().toDoubleOrNull() ?: 0.0
    }
  }

  private fun readAsStringList(target: Any, vararg names: String): List<String> {
    val value = readProperty(target, *names)
    return when (value) {
      is Iterable<*> -> value.filterNotNull().map { it.toString() }
      else -> emptyList()
    }
  }

  private fun readProperty(target: Any, vararg names: String): Any? {
    for (name in names) {
      val getterName = "get" + name.replaceFirstChar { if (it.isLowerCase()) it.titlecase() else it.toString() }
      val method = target.javaClass.methods.firstOrNull {
        it.parameterCount == 0 && (it.name == getterName || it.name == name)
      }
      if (method != null) {
        return method.invoke(target)
      }
    }
    return null
  }

  private fun toBalancesMap(balances: Any): WritableMap {
    val result = Arguments.createMap()
    result.putDouble("regular", readAsDouble(balances, "regular"))
    result.putDouble("swap", readAsDouble(balances, "swap"))
    result.putDouble("contract", readAsDouble(balances, "contract"))
    result.putDouble("fidelity", readAsDouble(balances, "fidelity"))
    result.putDouble("spendable", readAsDouble(balances, "spendable"))
    return result
  }

  private fun toSwapReportMap(report: Any): WritableMap {
    val result = Arguments.createMap()
    result.putString("swapId", readAsString(report, "swapId"))
    result.putDouble("outgoingAmount", readAsDouble(report, "outgoingAmount"))
    result.putDouble("incomingAmount", readAsDouble(report, "incomingAmount"))
    result.putDouble("feePaidOrEarned", readAsDouble(report, "feePaidOrEarned"))
    result.putDouble("makersCount", readAsDouble(report, "makersCount", "makerCount"))
    result.putDouble("totalMakerFees", readAsDouble(report, "totalMakerFees"))
    result.putDouble("miningFee", readAsDouble(report, "miningFee"))
    result.putString("status", readAsString(report, "status"))

    val makersArray = Arguments.createArray()
    readAsStringList(report, "makerAddresses").forEach(makersArray::pushString)
    result.putArray("makerAddresses", makersArray)

    return result
  }
}

private fun ReadableMap.getNullableString(key: String): String? {
  return if (hasKey(key) && !isNull(key)) getString(key) else null
}

private fun ReadableMap.getNullableMap(key: String): ReadableMap? {
  return if (hasKey(key) && !isNull(key)) getMap(key) else null
}

private fun ReadableMap.getNullableInt(key: String): Int? {
  return if (hasKey(key) && !isNull(key)) getDouble(key).toInt() else null
}

private fun ReadableMap.getRequiredString(key: String): String {
  return getNullableString(key) ?: throw IllegalArgumentException("Missing required string: $key")
}

private fun ReadableMap.getRequiredLong(key: String): Long {
  if (!hasKey(key) || isNull(key)) {
    throw IllegalArgumentException("Missing required number: $key")
  }
  return getDouble(key).toLong()
}

private fun ReadableMap.getRequiredInt(key: String): Int {
  if (!hasKey(key) || isNull(key)) {
    throw IllegalArgumentException("Missing required number: $key")
  }
  return getDouble(key).toInt()
}

private fun ReadableMap.getNullableStringArray(key: String): List<String>? {
  if (!hasKey(key) || isNull(key)) {
    return null
  }

  val value = getArray(key) ?: return null
  val list = mutableListOf<String>()
  for (i in 0 until value.size()) {
    if (value.getType(i) == ReadableType.String) {
      list.add(value.getString(i) ?: continue)
    }
  }
  return list
}
