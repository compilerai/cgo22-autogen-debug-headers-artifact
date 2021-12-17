#!/bin/bash
source $HOME/.cargo/env

# unpack support files
tar xvf eqfiles.tgz
tar xvf bin.tgz
# build evaluator
cd eval && cargo build && cd ..
# build dwarf-updater
cd gimli_write && cargo build && cd ..
