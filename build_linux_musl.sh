#!/usr/bin/env bash

Release=''

while (( "$#" )); do
  case "$1" in
  --release)
      Release='YES'
    ;;
  esac
  shift
done

if [ "$Release" == 'YES' ]; then
  docker run -v $PWD:/volume --rm -t clux/muslrust cargo build --release
  cp target/x86_64-unknown-linux-musl/release/codenamer ./musl_release
  strip musl_release &> /dev/null || true
else
  docker run -v $PWD:/volume --rm -t clux/muslrust cargo build
  cp target/x86_64-unknown-linux-musl/debug/codenamer ./musl_debug
fi
