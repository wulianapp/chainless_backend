# Chainless Backend Service

Modules
----------------------
- `api/src/account_manager`    some api  about user's login、register、captcha and user info
- `api/src/airdrop`            api of wrap airdrop contract interface
- `api/src/newbie_reward`      api of wrap newbie_reward contract interface
- `api/src/wallet`             wrapped multi-sig-contract
- `api/src/general`            other's common api
- `blockchain`             wrapped blockchain interface,
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

Api document
----------------
```
apidoc -f "mod.rs" -c tools/apidoc.json -i api/ -o docs/
```
