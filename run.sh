#!/bin/bash

set -eo pipefail

cargo build --bin back_end --target x86_64-unknown-linux-musl
sudo docker-compose up -d --build
trunk watch
