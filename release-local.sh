#!/bin/bash

cargo build --release
cp target/release/mkv-renamer ~/bin/
mkv-renamer -V
