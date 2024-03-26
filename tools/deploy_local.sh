#!/bin/bash

source ./tools/local.env
killall -9 api
cp ./target/debug/api ./target/debug/api_ori
nohup ./target/debug/api > ./api.log &
source ./tools/test.env

