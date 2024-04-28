## BlochChain

### 账户配置
- 按照 `环境模式`.`业务功能`.`账户类型` 的命名规则，比如 `local.multi_wallet.contract`
- 账户类型目前分为 `contract` 和 `relayer` 两种，前者是合约部署是代用户链上交互
- 部分账户需要上游合约配置白名单，需要白名单进行模糊匹配，例如许可包含字符串：`multi_wallet.contract`


| 账户                    | 功能               | 上游合约白名单函数 |
| ----------------------- | ----------------- | ---------------- |
| 待定(暂时是local和test）          | 链上地址创建    |   |
| bridge.relayer          | 桥交互  | cvault0003.chainless:set_user_batch、cvault0003.chainless:bind_eth_addr  |
| account_namange.relayer        | 主从设备管理、多签策略配置、手续费币种管理、子账户管理 | fees_call:set_fees_priority｜
| transfer.relayer | 子账户资金划转  |  |
| multi_wallet.contract   | 多签钱包合约    |cvault0003.chainless:new_order,btc:transfer_from_nongas  |