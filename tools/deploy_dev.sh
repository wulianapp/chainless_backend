#!/bin/bash
current_time=$(date +"%Y%m%d%H%M%S")
api_log_file="api_dev_${current_time}.log"

export CONFIG=/root/chainless_backend/config_dev.toml
killall -9 api_dev
cp ./target/debug/api  ./target/debug/api_dev
nohup ./target/debug/api_dev > ./$api_log_file &
export CONFIG=/root/chainless_backend/config_test.toml

