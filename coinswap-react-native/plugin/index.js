const { createRunOncePlugin, withPlugins } = require('@expo/config-plugins')

const { withBinaryArtifacts } = require('./withBinaryArtifacts')
const { withCoinswapAndroid } = require('./withAndroid')
const { sdkPackage } = require('./utils')

function withCoinswap(config, options) {
  const { skipBinaryDownload = false } = options || {}

  return withPlugins(config, [
    ...(skipBinaryDownload ? [] : [withBinaryArtifacts]),
    withCoinswapAndroid,
  ])
}

module.exports = createRunOncePlugin(withCoinswap, sdkPackage.name, sdkPackage.version)
