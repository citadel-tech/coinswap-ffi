const { withGradleProperties } = require('@expo/config-plugins')

function withCoinswapAndroid(config) {
  return withGradleProperties(config, (projectConfig) => {
    projectConfig.modResults = projectConfig.modResults.filter(
      (item) => item.type !== 'property' || item.key !== 'android.useAndroidX'
    )

    projectConfig.modResults.push({
      type: 'property',
      key: 'android.useAndroidX',
      value: 'true',
    })

    const hasNewArchEnabled = projectConfig.modResults.some(
      (item) => item.type === 'property' && item.key === 'newArchEnabled'
    )

    if (!hasNewArchEnabled) {
      projectConfig.modResults.push({
        type: 'property',
        key: 'newArchEnabled',
        value: 'true',
      })
    }

    return projectConfig
  })
}

module.exports = {
  withCoinswapAndroid,
}