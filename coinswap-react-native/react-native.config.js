module.exports = {
  dependency: {
    platforms: {
      android: {
        packageImportPath: 'import org.coinswap.reactnative.CoinswapPackage;',
        packageInstance: 'new CoinswapPackage()',
      },
      ios: {},
    },
  },
}
