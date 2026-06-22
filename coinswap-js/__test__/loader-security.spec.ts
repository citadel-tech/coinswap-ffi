import { execFileSync } from 'node:child_process'
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'

import test from 'ava'

import {
  assertTrustedModulePath,
  EXPECTED_BINDING_VERSION,
  requireOptionalBinding,
  tryLoadNativeLibraryPathOverride,
} from '../loader-security'

const packageRoot = join(fileURLToPath(new URL('..', import.meta.url)))
const hardenScript = join(packageRoot, 'scripts', 'harden-loader.js')
const packageVersion = JSON.parse(readFileSync(join(packageRoot, 'package.json'), 'utf8')).version

let tempDir: string

test.before(() => {
  tempDir = mkdtempSync(join(tmpdir(), 'coinswap-loader-security-'))
})

test.after(() => {
  rmSync(tempDir, { recursive: true, force: true })
})

test('EXPECTED_BINDING_VERSION matches package.json', (t) => {
  t.is(EXPECTED_BINDING_VERSION, packageVersion)
})

test('index.js passes hardening verification', (t) => {
  t.notThrows(() => {
    execFileSync(process.execPath, [hardenScript, '--verify'], { cwd: packageRoot, encoding: 'utf8' })
  })
})

test('assertTrustedModulePath rejects paths outside node_modules', (t) => {
  t.throws(
    () => assertTrustedModulePath('/tmp/fake/coinswap-napi-linux-x64-gnu/index.js', 'coinswap-napi-linux-x64-gnu'),
    { message: /Refusing to load native binding/ },
  )
})

test('assertTrustedModulePath accepts hoisted node_modules layout', (t) => {
  t.notThrows(() =>
    assertTrustedModulePath(
      '/app/node_modules/coinswap-napi-linux-x64-gnu/index.js',
      'coinswap-napi-linux-x64-gnu',
    ),
  )
})

test('tryLoadNativeLibraryPathOverride ignores path without opt-in', (t) => {
  const payloadPath = join(tempDir, 'payload.js')
  writeFileSync(payloadPath, 'global.__PAYLOAD_RAN = true\nmodule.exports = {}')

  const previousPath = process.env.NAPI_RS_NATIVE_LIBRARY_PATH
  const previousAllow = process.env.NAPI_RS_ALLOW_UNSAFE_NATIVE_PATH
  process.env.NAPI_RS_NATIVE_LIBRARY_PATH = payloadPath
  delete process.env.NAPI_RS_ALLOW_UNSAFE_NATIVE_PATH

  t.is(tryLoadNativeLibraryPathOverride(packageRoot), null)

  process.env.NAPI_RS_NATIVE_LIBRARY_PATH = previousPath
  if (previousAllow === undefined) {
    delete process.env.NAPI_RS_ALLOW_UNSAFE_NATIVE_PATH
  } else {
    process.env.NAPI_RS_ALLOW_UNSAFE_NATIVE_PATH = previousAllow
  }
})

test('tryLoadNativeLibraryPathOverride rejects path outside package directory', (t) => {
  const payloadPath = join(tempDir, 'outside-payload.js')
  writeFileSync(payloadPath, 'module.exports = {}')

  const previousPath = process.env.NAPI_RS_NATIVE_LIBRARY_PATH
  const previousAllow = process.env.NAPI_RS_ALLOW_UNSAFE_NATIVE_PATH
  process.env.NAPI_RS_NATIVE_LIBRARY_PATH = payloadPath
  process.env.NAPI_RS_ALLOW_UNSAFE_NATIVE_PATH = '1'

  t.throws(() => tryLoadNativeLibraryPathOverride(packageRoot), {
    message: /outside package directory/,
  })

  process.env.NAPI_RS_NATIVE_LIBRARY_PATH = previousPath
  if (previousAllow === undefined) {
    delete process.env.NAPI_RS_ALLOW_UNSAFE_NATIVE_PATH
  } else {
    process.env.NAPI_RS_ALLOW_UNSAFE_NATIVE_PATH = previousAllow
  }
})

test('import ignores NAPI_RS_NATIVE_LIBRARY_PATH without opt-in', (t) => {
  t.timeout(10_000)
  const payloadPath = join(tempDir, 'import-payload.js')
  writeFileSync(payloadPath, 'global.__PAYLOAD_RAN = true\nmodule.exports = {}')

  const env: NodeJS.ProcessEnv = { ...process.env, NAPI_RS_NATIVE_LIBRARY_PATH: payloadPath }
  delete env.NAPI_RS_ALLOW_UNSAFE_NATIVE_PATH

  const output = execFileSync(
    process.execPath,
    [
      '-e',
      `try { require(${JSON.stringify(join(packageRoot, 'index.js'))}) } catch {} console.log(global.__PAYLOAD_RAN ? 'ran' : 'safe')`,
    ],
    {
      cwd: packageRoot,
      env,
      encoding: 'utf8',
    },
  )

  t.is(output.trim(), 'safe')
})

test('requireOptionalBinding ignores NODE_PATH hijack', (t) => {
  t.timeout(10_000)
  const fakeRoot = join(tempDir, 'fake-node-path')
  const fakeModuleDir = join(fakeRoot, 'coinswap-napi-linux-x64-gnu')
  mkdirSync(fakeModuleDir, { recursive: true })
  writeFileSync(join(fakeModuleDir, 'package.json'), JSON.stringify({ version: '1.0.0' }))
  writeFileSync(join(fakeModuleDir, 'index.js'), 'module.exports = { hijacked: true }')

  const previousNodePath = process.env.NODE_PATH
  process.env.NODE_PATH = fakeRoot

  t.throws(
    () => requireOptionalBinding('coinswap-napi-linux-x64-gnu', packageRoot),
    { message: /Refusing to load native binding|Cannot find module/ },
  )

  if (previousNodePath === undefined) {
    delete process.env.NODE_PATH
  } else {
    process.env.NODE_PATH = previousNodePath
  }
})

test('index.js does not shell out to ldd on import', (t) => {
  t.timeout(10_000)
  const script = `
    const cp = require('node:child_process')
    const original = cp.execSync
    cp.execSync = (...args) => {
      if (String(args[0]).includes('ldd')) {
        throw new Error('ldd exec blocked')
      }
      return original.apply(cp, args)
    }
    try { require(${JSON.stringify(join(packageRoot, 'index.js'))}) } catch {}
    console.log('ok')
  `

  const output = execFileSync(process.execPath, ['-e', script], {
    cwd: packageRoot,
    encoding: 'utf8',
  })

  t.is(output.trim(), 'ok')
})
