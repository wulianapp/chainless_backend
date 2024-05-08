#!/bin/bash
current_time=$(date +"%Y%m%d%H%M%S")
api_log_file="api_${current_time}.log"
scanner_wallet_manage_log_file="wallet_manage_${current_time}.log"
scanner_coin_transfer_log_file="coin_transfer_${current_time}.log"
scanner_eth_bridge_log_file="eth_bridge_${current_time}.log"

source ./tools/local.env
killall -9 api
killall -9 scanner
cp ./target/debug/api ./target/debug/api_ori
cp ./target/debug/scanner ./target/debug/scanner_ori
nohup ./target/debug/api > ./$api_log_file &
nohup ./target/debug/scanner --task chainless_wallet_manage > ./$scanner_wallet_manage_log_file &
nohup ./target/debug/scanner --task chainless_coin_transfer > ./$scanner_coin_transfer_log_file &
nohup ./target/debug/scanner --task eth_bridge > ./$scanner_eth_bridge_log_file &

source ./tools/test.env

