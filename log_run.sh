#!/bin/bash

set euo pipefail

export RUST_LOG=smolbel=trace
cargo run
