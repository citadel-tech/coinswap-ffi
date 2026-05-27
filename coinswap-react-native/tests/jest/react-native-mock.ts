const installer = {
  installRustCrate() {
    return true
  },
  cleanupRustCrate() {
    return true
  },
}

export const TurboModuleRegistry = {
  getEnforcing: (moduleName: string) => {
    if (moduleName !== 'CoinswapReactNative') {
      throw new Error(`TurboModule not found: ${moduleName}`)
    }
    return installer
  },
  get: (moduleName: string) => (moduleName === 'CoinswapReactNative' ? installer : null),
}
