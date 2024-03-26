#!/bin/bash

source ./tools/test.env
cargo test test_wallet -- --nocapture --test-threads=1

