#!/bin/sh
echo 'Building'
cargo test --no-run 1>/dev/null
echo 'Cleaning'
rm -rf target/cov
echo 'Testing'
kcov --exclude-pattern=/.cargo target/cov target/debug/$(ls -t target/debug | grep mixer_wrappers- | grep -ve '\.d$' | head -1)
echo 'Test coverage results available at target/cov/index.html'
