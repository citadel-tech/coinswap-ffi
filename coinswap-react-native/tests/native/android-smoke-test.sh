#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
TMP_DIR=$(mktemp -d /private/tmp/coinswap-rn-android-smoke.XXXXXX)
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p \
  "$TMP_DIR/src/androidx/annotation" \
  "$TMP_DIR/src/com/facebook/react/bridge" \
  "$TMP_DIR/src/com/facebook/react/module/annotations" \
  "$TMP_DIR/src/com/facebook/react/module/model" \
  "$TMP_DIR/src/com/facebook/react/turbomodule/core/interfaces" \
  "$TMP_DIR/src/com/facebook/react" \
  "$TMP_DIR/src/org/coinswap/reactnative"

cat > "$TMP_DIR/src/androidx/annotation/NonNull.java" <<'JAVA'
package androidx.annotation;
public @interface NonNull {}
JAVA

cat > "$TMP_DIR/src/androidx/annotation/Nullable.java" <<'JAVA'
package androidx.annotation;
public @interface Nullable {}
JAVA

cat > "$TMP_DIR/src/com/facebook/react/module/annotations/ReactModule.java" <<'JAVA'
package com.facebook.react.module.annotations;
public @interface ReactModule { String name(); }
JAVA

cat > "$TMP_DIR/src/com/facebook/react/turbomodule/core/interfaces/CallInvokerHolder.java" <<'JAVA'
package com.facebook.react.turbomodule.core.interfaces;
public interface CallInvokerHolder {}
JAVA

cat > "$TMP_DIR/src/com/facebook/react/bridge/NativeModule.java" <<'JAVA'
package com.facebook.react.bridge;
public interface NativeModule {}
JAVA

cat > "$TMP_DIR/src/com/facebook/react/bridge/JavaScriptContextHolder.java" <<'JAVA'
package com.facebook.react.bridge;
public class JavaScriptContextHolder {
  public long get() { return 0L; }
}
JAVA

cat > "$TMP_DIR/src/com/facebook/react/bridge/CatalystInstance.java" <<'JAVA'
package com.facebook.react.bridge;
import com.facebook.react.turbomodule.core.interfaces.CallInvokerHolder;
public class CatalystInstance {
  public CallInvokerHolder getJSCallInvokerHolder() { return null; }
}
JAVA

cat > "$TMP_DIR/src/com/facebook/react/bridge/ReactApplicationContext.java" <<'JAVA'
package com.facebook.react.bridge;
public class ReactApplicationContext {
  public JavaScriptContextHolder getJavaScriptContextHolder() { return new JavaScriptContextHolder(); }
  public CatalystInstance getCatalystInstance() { return new CatalystInstance(); }
}
JAVA

cat > "$TMP_DIR/src/com/facebook/react/module/model/ReactModuleInfo.java" <<'JAVA'
package com.facebook.react.module.model;
public class ReactModuleInfo {
  public ReactModuleInfo(String name, String className, boolean canOverrideExistingModule, boolean needsEagerInit, boolean hasConstants, boolean isCxxModule, boolean isTurboModule) {}
}
JAVA

cat > "$TMP_DIR/src/com/facebook/react/module/model/ReactModuleInfoProvider.java" <<'JAVA'
package com.facebook.react.module.model;
import java.util.Map;
public interface ReactModuleInfoProvider {
  Map<String, ReactModuleInfo> getReactModuleInfos();
}
JAVA

cat > "$TMP_DIR/src/com/facebook/react/TurboReactPackage.java" <<'JAVA'
package com.facebook.react;
import com.facebook.react.bridge.NativeModule;
import com.facebook.react.bridge.ReactApplicationContext;
import com.facebook.react.module.model.ReactModuleInfoProvider;
public abstract class TurboReactPackage {
  public abstract NativeModule getModule(String name, ReactApplicationContext reactContext);
  public abstract ReactModuleInfoProvider getReactModuleInfoProvider();
}
JAVA

cat > "$TMP_DIR/src/org/coinswap/reactnative/NativeCoinswapReactNativeSpec.java" <<'JAVA'
package org.coinswap.reactnative;
import com.facebook.react.bridge.NativeModule;
import com.facebook.react.bridge.ReactApplicationContext;
public abstract class NativeCoinswapReactNativeSpec implements NativeModule {
  private final ReactApplicationContext context;

  protected NativeCoinswapReactNativeSpec(ReactApplicationContext reactContext) {
    this.context = reactContext;
  }

  protected ReactApplicationContext getReactApplicationContext() {
    return this.context;
  }

  public abstract String getName();
  public abstract boolean installRustCrate();
  public abstract boolean cleanupRustCrate();
}
JAVA

javac -d "$TMP_DIR/out" \
  "$TMP_DIR/src/androidx/annotation/NonNull.java" \
  "$TMP_DIR/src/androidx/annotation/Nullable.java" \
  "$TMP_DIR/src/com/facebook/react/module/annotations/ReactModule.java" \
  "$TMP_DIR/src/com/facebook/react/turbomodule/core/interfaces/CallInvokerHolder.java" \
  "$TMP_DIR/src/com/facebook/react/bridge/NativeModule.java" \
  "$TMP_DIR/src/com/facebook/react/bridge/JavaScriptContextHolder.java" \
  "$TMP_DIR/src/com/facebook/react/bridge/CatalystInstance.java" \
  "$TMP_DIR/src/com/facebook/react/bridge/ReactApplicationContext.java" \
  "$TMP_DIR/src/com/facebook/react/module/model/ReactModuleInfo.java" \
  "$TMP_DIR/src/com/facebook/react/module/model/ReactModuleInfoProvider.java" \
  "$TMP_DIR/src/com/facebook/react/TurboReactPackage.java" \
  "$TMP_DIR/src/org/coinswap/reactnative/NativeCoinswapReactNativeSpec.java" \
  "$ROOT_DIR/android/src/main/java/org/coinswap/reactnative/CoinswapReactNativeModule.java" \
  "$ROOT_DIR/android/src/main/java/org/coinswap/reactnative/CoinswapReactNativePackage.java"

echo "Android smoke test passed: Java bridge classes compile against expected RN interfaces."
