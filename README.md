# coinswap-ffi
FFI layer for the coinswap client

## APIs required according to the taker-app

### Wallet
- [x] get_balances
- [x] list_all_utxos (spendinfo, listutxo) -> wallet api
- [ ] list_transactions -> rpc

### Market
- [x] Makers Info (Avg fee?, Total Liquidity?, Online Makers, Avg Response time?) -> taker api/display_offer, taker/offers.rs -> all_good_makers
- [ ] Min and Max Size -> all_good_makers
- [ ] Bond Amount -> all_good_makers
- [ ] Fee (Timelock PCT, absolute PCT, maker fee) -> all_good_makers
- [ ] Bad Makers -> taker api

### Send
- [ ] Network Fee (Dynamic, realtime) -> ???
- [ ] API for getting different currencies meh translation -> external api

### Receive
- [ ] Recent_Addresses?

### Swap
- [ ] Number of Maker (Online Makers) -> all_good_makers.len()