import { execFileSync } from 'node:child_process'
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'

import test from 'ava'

import {
  assertTrustedModulePath,
  EXPECTED_BINDING_VERSION,
  tryLoadNativeLibraryPathOverride,
} from '../loader-security'

const packageRoot = join(fileURLToPath(new URL('..', import.meta.url)))
const hardenScript = join(packageRoot, 'scripts', 'harden-loader.js')
const packageVersion = JSON.parse(readFileSync(join(packageRoot, 'package.json'), 'utf8')).version

let tempDir: string

function withEnv(overrides: Record<string, string | undefined>, fn: () => void) {
  const keys = new Set([...Object.keys(process.env), ...Object.keys(overrides)])
  const snapshot = new Map<string, string | undefined>()
  for (const key of keys) {
    snapshot.set(key, process.env[key])
  }

  for (const [key, value] of Object.entries(overrides)) {
    if (value === undefined) {
      delete process.env[key]
    } else {
      process.env[key] = value
    }
  }

  try {
    fn()
  } finally {
    for (const [key, value] of snapshot) {
      if (value === undefined) {
        delete process.env[key]
      } else {
        process.env[key] = value
      }
    }
  }
}

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

test('assertTrustedModulePath rejects paths outside trusted node_modules roots', (t) => {
  t.throws(
    () =>
      assertTrustedModulePath(
        '/tmp/fake/coinswap-napi-linux-x64-gnu/index.js',
        'coinswap-napi-linux-x64-gnu',
        packageRoot,
      ),
    { message: /Refusing to load native binding/ },
  )
})

test('assertTrustedModulePath accepts hoisted node_modules layout', (t) => {
  t.notThrows(() =>
    assertTrustedModulePath(
      '/app/node_modules/coinswap-napi-linux-x64-gnu/index.js',
      'coinswap-napi-linux-x64-gnu',
      '/app/node_modules/coinswap-napi',
    ),
  )
})

test.serial('tryLoadNativeLibraryPathOverride ignores path without opt-in', (t) => {
  const payloadPath = join(tempDir, 'payload.js')
  writeFileSync(payloadPath, 'global.__PAYLOAD_RAN = true\nmodule.exports = {}')

  withEnv(
    {
      NAPI_RS_NATIVE_LIBRARY_PATH: payloadPath,
      NAPI_RS_ALLOW_UNSAFE_NATIVE_PATH: undefined,
    },
    () => {
      t.is(tryLoadNativeLibraryPathOverride(packageRoot), null)
    },
  )
})

test.serial('tryLoadNativeLibraryPathOverride rejects path outside package directory', (t) => {
  const payloadPath = join(tempDir, 'outside-payload.js')
  writeFileSync(payloadPath, 'module.exports = {}')

  withEnv(
    {
      NAPI_RS_NATIVE_LIBRARY_PATH: payloadPath,
      NAPI_RS_ALLOW_UNSAFE_NATIVE_PATH: '1',
    },
    () => {
      t.throws(() => tryLoadNativeLibraryPathOverride(packageRoot), {
        message: /outside package directory/,
      })
    },
  )
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

test('requireOptionalBinding ignores NODE_PATH hijack at process startup', (t) => {
  t.timeout(10_000)
  const fakeRoot = join(tempDir, 'fake-node-path')
  const fakeModuleDir = join(fakeRoot, 'coinswap-napi-linux-x64-gnu')
  mkdirSync(fakeModuleDir, { recursive: true })
  writeFileSync(join(fakeModuleDir, 'package.json'), JSON.stringify({ version: '1.0.0' }))
  writeFileSync(join(fakeModuleDir, 'index.js'), 'module.exports = { hijacked: true }')

  const loaderSecurityPath = join(packageRoot, 'loader-security.js')
  const script = `
    const { requireOptionalBinding } = require(${JSON.stringify(loaderSecurityPath)});
    try {
      requireOptionalBinding('coinswap-napi-linux-x64-gnu', ${JSON.stringify(packageRoot)});
      console.log('FAIL');
    } catch {
      console.log('PASS');
    }
  `

  const output = execFileSync(process.execPath, ['-e', script], {
    cwd: packageRoot,
    env: { ...process.env, NODE_PATH: fakeRoot },
    encoding: 'utf8',
  })

  t.is(output.trim(), 'PASS')
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
