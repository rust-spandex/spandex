#!/usr/bin/env bash

cargo doc --no-deps --lib
git clone https://github.com/rust-spandex/book/
cd book
mdbook build
cd book
cp -r ../../target/doc/* .

