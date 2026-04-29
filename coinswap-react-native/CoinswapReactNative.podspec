Pod::Spec.new do |s|
  s.name         = 'coinswap-react-native'
  s.version      = '1.0.0'
  s.summary      = 'React Native TurboModule bindings for Coinswap'
  s.description  = 'React Native TurboModule wrapper around Coinswap UniFFI bindings.'
  s.homepage     = 'https://github.com/citadel-tech/coinswap-ffi'
  s.license      = 'MIT'
  s.authors      = { 'Citadel-Tech' => 'dev@citadel-tech.org' }
  s.source       = { :path => '.' }

  s.platforms    = { :ios => '13.0' }
  s.swift_version = '5.7'
  s.static_framework = true

  s.source_files = 'ios/**/*.{h,m,mm,swift}', 'ios/generated/**/*.swift'
  s.vendored_frameworks = 'ios/coinswap_ffi.xcframework'

  s.dependency 'React-Core'
  s.dependency 'React-Codegen'

  s.pod_target_xcconfig = {
    'DEFINES_MODULE' => 'YES',
    'CLANG_CXX_LANGUAGE_STANDARD' => 'c++17'
  }
end
