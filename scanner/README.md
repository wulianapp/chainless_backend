
## 监控链上行为

### run
```
//监控用户内部转账、记录、重试
./target/debug/scanner --task chainless_wallet_manage

//监控设备管理操作、记录、重试
./target/debug/scanner --task chainless_coin_transfer

//监控用户签名的操作，记录状态
./target/debug/scanner --task eth_bridge
```
