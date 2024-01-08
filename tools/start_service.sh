#!/bin/sh
nohup ../target/release/api  > api_dev.log 2>&1 &
nohup ../target/release/launch  > launch_dev.log 2>&1 &
nohup ../target/release/ws  > ws_dev.log 2>&1 &
nohup ../target/release/scanner  > scanner_dev.log 2>&1 &
nohup ../target/release/engine WBTC-USDT  > engine_WBTC_USDT_dev.log 2>&1 &
nohup ../target/release/engine WETH-USDT  > engine_WETH_USDT_dev.log 2>&1 &
nohup ../target/release/engine CEC-USDT  > engine_CEC_USDT_dev.log 2>&1 &
nohup ../target/release/engine WBTC-CEC  > engine_WBTC_CEC_dev.log 2>&1 &
nohup ../target/release/engine WETH-CEC  > engine_WETH_CEC_dev.log 2>&1 &
