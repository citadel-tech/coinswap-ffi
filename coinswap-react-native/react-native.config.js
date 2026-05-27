module.exports = {
  dependency: {
    platforms: {
      android: {
        packageImportPath: 'import org.coinswap.reactnative.CoinswapReactNativePackage;',
        packageInstance: 'new CoinswapReactNativePackage()',
      },
      ios: {},
    },
  },
}
