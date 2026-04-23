package org.coinswap.reactnative

import com.facebook.react.TurboReactPackage
import com.facebook.react.bridge.NativeModule
import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.module.model.ReactModuleInfo
import com.facebook.react.module.model.ReactModuleInfoProvider

class CoinswapPackage : TurboReactPackage() {
  override fun getModule(name: String, reactContext: ReactApplicationContext): NativeModule? {
    return if (name == CoinswapModule.NAME) {
      CoinswapModule(reactContext)
    } else {
      null
    }
  }

  override fun getReactModuleInfoProvider(): ReactModuleInfoProvider {
    return ReactModuleInfoProvider {
      mapOf(
        CoinswapModule.NAME to ReactModuleInfo(
          CoinswapModule.NAME,
          CoinswapModule.NAME,
          false,
          false,
          false,
          false,
          true,
        ),
      )
    }
  }
}
