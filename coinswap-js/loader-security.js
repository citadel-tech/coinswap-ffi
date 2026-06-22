'use strict'

const path = require('node:path')

const { version: EXPECTED_BINDING_VERSION } = require('./package.json')

function isPathInside(resolvedPath, root) {
  const normalized = path.resolve(resolvedPath)
  const normalizedRoot = path.resolve(root)
  return normalized === normalizedRoot || normalized.startsWith(`${normalizedRoot}${path.sep}`)
}

function trustedNodeModulesRoots(fromDir) {
  const roots = []
  let dir = path.resolve(fromDir)
  while (true) {
    roots.push(path.join(dir, 'node_modules'))
    const parent = path.dirname(dir)
    if (parent === dir) {
      break
    }
    dir = parent
  }
  return roots
}

/**
 * If platform bindings are ever published as scoped packages (e.g.
 * `@coinswap/napi-linux-x64-gnu`), update this assertion for scoped layouts.
 */
function assertTrustedModulePath(resolvedPath, moduleName, fromDir) {
  const normalized = path.normalize(resolvedPath)
  const inNodeModules = `${path.sep}node_modules${path.sep}${moduleName}${path.sep}`
  const atNodeModulesRoot = normalized.endsWith(`${path.sep}node_modules${path.sep}${moduleName}`)

  if (!normalized.includes(inNodeModules) && !atNodeModulesRoot) {
    throw new Error(
      `[coinswap-napi] Refusing to load native binding '${moduleName}' from untrusted path.\n` +
        `  Resolved to: ${resolvedPath}\n` +
        `  Expected path to contain: node_modules${path.sep}${moduleName}\n` +
        `  This may indicate a NODE_PATH override or an installation issue.\n` +
        `  Try: yarn install --force`,
    )
  }

  const trustedPackageRoots = trustedNodeModulesRoots(fromDir).map((root) => path.join(root, moduleName))
  if (!trustedPackageRoots.some((pkgRoot) => isPathInside(normalized, pkgRoot))) {
    throw new Error(
      `[coinswap-napi] Refusing to load native binding '${moduleName}' from untrusted path.\n` +
        `  Resolved to: ${resolvedPath}\n` +
        `  Expected path under a node_modules tree reachable from: ${path.resolve(fromDir)}\n` +
        `  This may indicate a NODE_PATH override or an installation issue.\n` +
        `  Try: yarn install --force`,
    )
  }
}

function enforceVersionIfRequired(bindingPackageVersion) {
  if (
    bindingPackageVersion !== EXPECTED_BINDING_VERSION &&
    process.env.NAPI_RS_ENFORCE_VERSION_CHECK &&
    process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0'
  ) {
    throw new Error(
      `Native binding package version mismatch, expected ${EXPECTED_BINDING_VERSION} but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`,
    )
  }
}

function requireOptionalBinding(moduleName, fromDir) {
  const bindingPath = require.resolve(moduleName, { paths: [fromDir] })
  assertTrustedModulePath(bindingPath, moduleName, fromDir)

  const pkgJsonPath = require.resolve(`${moduleName}/package.json`, { paths: [fromDir] })
  assertTrustedModulePath(pkgJsonPath, moduleName, fromDir)

  enforceVersionIfRequired(require(pkgJsonPath).version)
  return require(bindingPath)
}

function tryLoadNativeLibraryPathOverride(fromDir) {
  const overridePath = process.env.NAPI_RS_NATIVE_LIBRARY_PATH
  if (!overridePath || process.env.NAPI_RS_ALLOW_UNSAFE_NATIVE_PATH !== '1') {
    return null
  }

  if (!path.isAbsolute(overridePath)) {
    throw new Error('NAPI_RS_NATIVE_LIBRARY_PATH must be an absolute path')
  }

  const pkgRoot = path.resolve(fromDir)
  const resolvedOverride = path.resolve(overridePath)
  if (resolvedOverride !== pkgRoot && !resolvedOverride.startsWith(`${pkgRoot}${path.sep}`)) {
    throw new Error(
      `Refusing to load native binding from outside package directory: ${resolvedOverride}`,
    )
  }

  return require(resolvedOverride)
}

const isFileMusl = (f) => f.includes('libc.musl-') || f.includes('ld-musl-')

function isMuslFromFilesystem(readFileSync) {
  try {
    return readFileSync('/usr/bin/ldd', 'utf-8').includes('musl')
  } catch {
    return null
  }
}

function isMuslFromReport() {
  let report = null
  if (typeof process.report?.getReport === 'function') {
    process.report.excludeNetwork = true
    report = process.report.getReport()
  }
  if (!report) {
    return null
  }
  if (report.header && report.header.glibcVersionRuntime) {
    return false
  }
  if (Array.isArray(report.sharedObjects) && report.sharedObjects.some(isFileMusl)) {
    return true
  }
  return false
}

function isMusl(readFileSync) {
  if (process.platform !== 'linux') {
    return false
  }

  let musl = isMuslFromFilesystem(readFileSync)
  if (musl === null) {
    musl = isMuslFromReport()
  }
  return musl ?? false
}

module.exports = {
  EXPECTED_BINDING_VERSION,
  assertTrustedModulePath,
  isMusl,
  requireOptionalBinding,
  tryLoadNativeLibraryPathOverride,
}
