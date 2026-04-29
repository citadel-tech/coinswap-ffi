const fs = require('fs')
const path = require('path')

const { sdkPackage } = require('./utils')

function withBinaryArtifacts(config) {
  downloadBinaryArtifacts()
  return config
}

function downloadBinaryArtifacts() {
  const packageRoot = findPackageRoot()
  if (!packageRoot) {
    console.warn(`Could not find ${sdkPackage.name} package root while preparing React Native artifacts.`)
    return
  }

  const androidLibsPath = path.join(packageRoot, 'android/src/main/jniLibs')
  const iosFrameworkPath = path.join(packageRoot, 'ios/coinswap_ffi.xcframework')

  if (fs.existsSync(androidLibsPath) && fs.existsSync(iosFrameworkPath)) {
    return
  }

  console.warn(
    'Coinswap React Native native artifacts are not present yet. Run the package build scripts before creating an Expo project or release build.'
  )
}

function findPackageRoot() {
  try {
    const resolved = require.resolve('../package.json')
    return path.dirname(resolved)
  } catch {
    return path.resolve(__dirname, '..')
  }
}

module.exports = {
  withBinaryArtifacts,
}