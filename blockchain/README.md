## BlochChain

### 官方账户配置
- 按照 `环境模式`.`业务功能`.`账户类型` 的命名规则，比如 `local.multi_wallet.contract`
- 上游白名单针对`multi_sig`,`backend_relayer_pool`模糊过滤加白


| 账户                    | 功能               | 上游合约白名单函数 |   例子                |
| ----------------------- | ----------------- | ---------------- |---------------------|
| local/dev/pro(1-1000)   | 用户链上地址创建  |  - | 官方为local765，用户为123abc.local765 |
| multi_sig          | 多签钱包合约  |  usdt::transfer_from_nongas cvault0003.chainless::new_order  |local.multi_sig_wallet
| backend_relayer_pool       | fees_call的设置手续费顺序 bridge的set_batch设置状态， 用主子账户资金划转 用户多签策略和设备管理 |  cvault0003.chainless:set_user_batch cvault0003.chainless:bind_eth_addr fees_call:set_fees_priority | dev.1.backend_relayer_pool dev.223.backend_relayer_pool