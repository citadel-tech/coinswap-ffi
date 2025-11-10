# coinswap-ffi
FFI layer for the coinswap client

## APIs required according to the taker-app

### Wallet
- [x] get_balances
- [x] list_all_utxos (spendinfo, listutxo) -> wallet api
- [ ] list_transactions -> rpc
- [ ] Send_to_address
- [x] Get new internal address (imp for above)
- [x] Get new external address

### Market
- [x] Makers Info (Avg fee?, Total Liquidity?, Online Makers, Avg Response time?) -> taker api/display_offer, taker/offers.rs -> all_good_makers
- [x] Min and Max Size -> all_good_makers
- [x] Bond Amount -> all_good_makers
- [x] Fee (Timelock PCT, absolute PCT, maker fee) -> all_good_makers
- [x] Bad Makers -> taker api

### Send
- [ ] Network Fee (Dynamic, realtime) -> ???
- [ ] API for getting different currencies meh translation -> external api

### Receive
- [ ] Recent_Addresses?

### Swap
- [x] Number of Maker (Online Makers) -> all_good_makers.len()
