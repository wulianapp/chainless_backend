#!/bin/bash
current_time=$(date +"%Y%m%d%H%M%S")
log_file="api_${current_time}.log"

source ./tools/local.env
killall -9 api
cp ./target/debug/api ./target/debug/api_ori
nohup ./target/debug/api > ./$log_file &
source ./tools/test.env

