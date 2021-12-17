#!/bin/bash

cd bin
ln -sf update_debug_headers_all update_debug_headers
# for clang
./runall.sh ../eqfiles/clang_proof_files/ ../eqfiles/TSVC_combined/ ../eqfiles/TSVC_combined/ ../eqfiles/TSVC_combined/ ../eqfiles/TSVC_combined/ results-clang-all.csv ../gimli_write/ ../eval/ clang rewrites_dir_all
# for gcc
./runall.sh ../eqfiles/gcc_proof_files/ ../eqfiles/TSVC_combined/ ../eqfiles/TSVC_combined/ ../eqfiles/TSVC_combined/ ../eqfiles/TSVC_combined/ results-gcc-all.csv ../gimli_write/ ../eval/ gcc rewrites_dir_all
# for icc
./runall.sh ../eqfiles/icc_proof_files/ ../eqfiles/TSVC_combined/ ../eqfiles/TSVC_combined/ ../eqfiles/TSVC_combined/ ../eqfiles/TSVC_combined/ results-icc-all.csv ../gimli_write/ ../eval/ icc rewrites_dir_all
