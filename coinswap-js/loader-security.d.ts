export declare const EXPECTED_BINDING_VERSION: string

export declare function assertTrustedModulePath(resolvedPath: string, moduleName: string): void

export declare function requireOptionalBinding(moduleName: string, fromDir: string): unknown

export declare function tryLoadNativeLibraryPathOverride(fromDir: string): unknown

export declare function isMusl(readFileSync: typeof import('node:fs').readFileSync): boolean
