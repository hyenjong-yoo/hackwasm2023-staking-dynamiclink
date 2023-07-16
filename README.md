## Building

```sh
  $ cargo wasm
```

## Testing

```sh
  $ cargo unit-test
```

## Compiling

```sh
  $ docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 .
```

## Demo

```sh
There are two contracts used for DEX (decentralized exchange) service implementation. You can consider 1. cw721 base contract as a helper contract for 2. dex contract
1. cw721 base contract : https://hw-scan.finschia.network/Finschia-Base/cosmwasm/49/contracts
2. dex contract : https://hw-scan.finschia.network/Finschia-Base/cosmwasm/50/contracts

Website: Finschia-Base : https://hw-scan.finschia.network/Finschia-Base

Note: Transactions (tx) were made on Finschia-Base in this demonstration

Features:
1. staking coin // tx : https://hw-scan.finschia.network/Finschia-Base/tx/37BE8B7B18A0C63E3CD246863DDD497B40094E09ECF8B8E64E28E7690037F3B3
- requester: link1l5895wgksj8pyzpwkle24n2jnl5ewvmzexm370 (user)
- requested to: link1lt0k03fwhdrlpm5asvvd2la9ef5yjdq4zru5tp928x73ydcaltmq55tk67 (dex contract)
- stake requested: 10ucony
- fee : 1ucony
- actual amount staked: 9ucony

2. reward claim with NFT // tx : https://hw-scan.finschia.network/Finschia-Base/tx/0C162AE93BFB60148B5A77EE94CAACAB4A4696B2560619FD8BD0B161CACEBC0E
- requester: link1l5895wgksj8pyzpwkle24n2jnl5ewvmzexm370 (user)
- requested to: link1lt0k03fwhdrlpm5asvvd2la9ef5yjdq4zru5tp928x73ydcaltmq55tk67 (dex contract)
- reward recieved: 24ucony (9ucony unstaked + 15ucony from block height difference)

3. transfer NFT // tx: https://hw-scan.finschia.network/Finschia-Base/tx/5F6674D86A15D45C8F6CC574DB99FF3DBA537565D1E6AAB5144DDD67C50BA026 
- requester: link1l5895wgksj8pyzpwkle24n2jnl5ewvmzexm370 (user)
- requested to: link1lt0k03fwhdrlpm5asvvd2la9ef5yjdq4zru5tp928x73ydcaltmq55tk67 (dex contract)
- token_id to transfer: cw721_290563
- transfer to whom: link15608kvpuz7a0cddcn4jp560yk0uwyjgys5hx8slare56wfdcasesrpr584

4.1 swap coin for the coin on another chain // tx (uCony to uBrown) : https://hw-scan.finschia.network/Finschia-Base/tx/4D8BED197FEDCCCCF6F6A6411997942F640B48836A285256F66EFC5B86E2A165 
- requester: link1l5895wgksj8pyzpwkle24n2jnl5ewvmzexm370 (user)
- requested to: link1lt0k03fwhdrlpm5asvvd2la9ef5yjdq4zru5tp928x73ydcaltmq55tk67 (dex contract)
- denom to give: uCony
- denom to receive: uBrown (ibc/04256FC86729F6ECC4A7C0EE915D33CD9C0C7596168F638F778EA6B59D7283E9)

4.2 swap coin for the coin on another chain // tx (uBrown to uCony) : https://hw-scan.finschia.network/Finschia-Base/tx/54FA7374762AEE4C712859EA5AE3A5AAD04DCF35A9A6BAB09D6A880F1BD4BFA9 
- requester: link1l5895wgksj8pyzpwkle24n2jnl5ewvmzexm370 (user)
- requested to: link1lt0k03fwhdrlpm5asvvd2la9ef5yjdq4zru5tp928x73ydcaltmq55tk67 (dex contract)
- denom to give: uBrown(ibc/04256F)
- denom to receive: uCony
```
