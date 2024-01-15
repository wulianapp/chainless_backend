# Chainless backend service

Modules
----------------------
- `api/account_manager`    some api  about user's login、register、verification_code and user info
- `api/airdrop`            api of wrap airdrop contract interface
- `api/newbie_reward`      api of wrap newbie_reward contract interface
- `api/wallet`             mpc-wallet`s message relayer,message notify、combine signature and so on
- `api/general`            other's common api
- `blockchain`             wrap blockchain interface,
- `scanner`                process for scan chain data ,and insert or update in database
- `modles`                 wrap postgres sql
- `common`                 wrap some reused func
- `doc`                    all api doc, [online address](http://120.232.251.101:8069/index.html)
- `tool`                   database operate and environment setup


Test
-----------
```
cargo test test_all_braced  -- --nocapture
```
