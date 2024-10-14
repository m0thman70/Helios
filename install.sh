#!/bin/bash
target=target/release/atto
bin=/usr/bin

cargo build --release
chmod +x $target
mv $target $bin
