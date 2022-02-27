#!/bin/bash

set euo pipefail

export RUST_LOG=smolbel=debug
cargo run
