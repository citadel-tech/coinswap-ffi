#import <React/RCTBridgeModule.h>
#import <React/RCTUtils.h>
#import <ReactCommon/RCTTurboModule.h>

#if __has_include("NativeCoinswapSpec.h")
#import "NativeCoinswapSpec.h"
#else
#import <NativeCoinswapSpec/NativeCoinswapSpec.h>
#endif

@interface RCT_EXTERN_MODULE(Coinswap, NSObject)

RCT_EXTERN_METHOD(setupLogging:(NSString * _Nullable)dataDir
                  level:(NSString *)level
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(createTaker:(NSDictionary *)config
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(disposeTaker:(NSString *)takerId
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(syncOfferbookAndWait:(NSString *)takerId
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(syncAndSave:(NSString *)takerId
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(getBalances:(NSString *)takerId
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(getNextExternalAddress:(NSString *)takerId
                  addressType:(NSString *)addressType
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(prepareCoinswap:(NSString *)takerId
                  swapParams:(NSDictionary *)swapParams
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(startCoinswap:(NSString *)takerId
                  swapId:(NSString *)swapId
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

@end

@implementation Coinswap (TurboModule)

- (std::shared_ptr<facebook::react::TurboModule>)getTurboModule:
    (const facebook::react::ObjCTurboModule::InitParams &)params {
  return std::make_shared<facebook::react::NativeCoinswapSpecJSI>(params);
}

@end
