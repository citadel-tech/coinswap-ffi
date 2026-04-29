const { createRunOncePlugin, withPlugins } = require('@expo/config-plugins')

const { withBinaryArtifacts } = require('./withBinaryArtifacts')
const { withCoinswapAndroid } = require('./withAndroid')
const { withCoinswapIOS } = require('./withIOS')
const { sdkPackage } = require('./utils')

function withCoinswap(config, options) {
  const { skipBinaryDownload = false } = options || {}

  return withPlugins(config, [
    ...(skipBinaryDownload ? [] : [withBinaryArtifacts]),
    withCoinswapAndroid,
    withCoinswapIOS,
  ])
}

module.exports = createRunOncePlugin(withCoinswap, sdkPackage.name, sdkPackage.version)