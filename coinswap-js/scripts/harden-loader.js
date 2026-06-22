#!/usr/bin/env node
'use strict'

const fs = require('node:fs')
const path = require('node:path')

const INDEX_PATH = path.join(__dirname, '..', 'index.js')
const HARDENED_MARKER = "require('./loader-security')"

const FS_IMPORT = /const \{ readFileSync \} = require\('(?:node:)?fs'\)\n/
const MUSL_BLOCK =
  /const isMusl = \(\) => \{[\s\S]*?const isMuslFromChildProcess = \(\) => \{[\s\S]*?\n\}\n\n/
const ENV_OVERRIDE =
  /  if \(process\.env\.NAPI_RS_NATIVE_LIBRARY_PATH\) \{\n    try \{\n      return require\(process\.env\.NAPI_RS_NATIVE_LIBRARY_PATH\);\n    \} catch \(err\) \{\n      loadErrors\.push\(err\)\n    \}\n  \} else if \(process\.platform === 'android'\) \{/
const OPTIONAL_BINDING =
  /(\s*)try \{\n\s*const binding = require\('(coinswap-napi[^']+)'\)\n\s*const bindingPackageVersion = require\('[^']+\/package\.json'\)\.version\n[\s\S]*?\n\s*return binding\n\1\} catch \(e\) \{\n\s*loadErrors\.push\(e\)\n\s*\}/g

const SECURITY_IMPORT = `const {
  isMusl: detectMusl,
  requireOptionalBinding,
  tryLoadNativeLibraryPathOverride,
} = require('./loader-security')
const isMusl = () => detectMusl(readFileSync)

`

const ENV_REPLACEMENT = `  try {
    const overrideBinding = tryLoadNativeLibraryPathOverride(__dirname)
    if (overrideBinding) {
      return overrideBinding
    }
  } catch (err) {
    loadErrors.push(err)
  }

  if (process.platform === 'android') {`

function isFullyHardened(source) {
  const required = [
    HARDENED_MARKER,
    'tryLoadNativeLibraryPathOverride',
    'requireOptionalBinding(',
    'detectMusl(readFileSync)',
  ]
  const forbidden = [
    'isMuslFromChildProcess',
    "require(process.env.NAPI_RS_NATIVE_LIBRARY_PATH)",
    "require('coinswap-napi-",
    "wasiBinding = require('coinswap-napi-wasm32-wasi')",
  ]
  return required.every((s) => source.includes(s)) && forbidden.every((s) => !source.includes(s))
}

function hardenIndexJs(source) {
  if (isFullyHardened(source)) {
    return { changed: false, source, optionalBindingCount: 0 }
  }

  let result = source
  if (result.includes('isMuslFromChildProcess')) {
    result = result.replace(MUSL_BLOCK, '')
  }
  if (!result.includes(HARDENED_MARKER)) {
    if (!FS_IMPORT.test(result)) {
      throw new Error('harden-loader: could not find readFileSync import in index.js')
    }
    result = result.replace(FS_IMPORT, (m) => `${m}${SECURITY_IMPORT}`)
  }
  if (result.includes('process.env.NAPI_RS_NATIVE_LIBRARY_PATH')) {
    result = result.replace(ENV_OVERRIDE, ENV_REPLACEMENT)
  }

  let optionalBindingCount = 0
  result = result.replace(OPTIONAL_BINDING, (_m, outerIndent, moduleName) => {
    optionalBindingCount++
    const inner = `${outerIndent}  `
    return `${outerIndent}try {
${inner}return requireOptionalBinding('${moduleName}', __dirname)
${outerIndent}} catch (e) {
${inner}loadErrors.push(e)
${outerIndent}}`
  })

  if (result.includes("wasiBinding = require('coinswap-napi-wasm32-wasi')")) {
    result = result.replace(
      "wasiBinding = require('coinswap-napi-wasm32-wasi')",
      "wasiBinding = requireOptionalBinding('coinswap-napi-wasm32-wasi', __dirname)",
    )
  }

  if (!isFullyHardened(result)) {
    throw new Error('harden-loader: index.js is still missing required hardening after patch')
  }

  return { changed: result !== source, source: result, optionalBindingCount }
}

function main() {
  const verifyOnly = process.argv.includes('--verify')
  const source = fs.readFileSync(INDEX_PATH, 'utf8')

  if (verifyOnly) {
    if (!isFullyHardened(source)) {
      console.error('harden-loader: index.js is not hardened — run: node scripts/harden-loader.js')
      process.exit(1)
    }
    console.log('harden-loader: index.js is hardened')
    return
  }

  const { changed, source: hardened, optionalBindingCount } = hardenIndexJs(source)
  if (changed) {
    fs.writeFileSync(INDEX_PATH, hardened)
    console.log(`harden-loader: patched index.js (${optionalBindingCount} optional-binding blocks)`)
    console.log('harden-loader: wrote', INDEX_PATH)
  } else {
    console.log('harden-loader: no changes needed')
  }
}

main()
