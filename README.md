# Artifact for "Automatic Generation of Debug Headers through BlackBox Equivalence Checking"

The artifact contains source code for our tool, along with shell scripts
and a dockerfile to install it inside a docker container.  It also contains the output
files from an equivalence checker (the .proof files) that are taken as input by our tool, along with the
source and object files for TSVC benchmarking suite compiled with three different compilers -- gcc,
clang/llvm and icc.  It can be used to reproduce the experimental results in Tables 2 and 3 of
our CGO'22 paper -- **Automatic Generation of Debug Headers through BlackBox Equivalence Checking**.
To validate the results, simply run the shell scripts and check the respective CSV output files
for each compiler.

## Artifact check-list (meta-information)

  - **Algorithm:** Identifying the mappings between source variables and the assembly locations
  - **Program:** Rust code
  - **Compilation:** rustc >= 1.41.0
  - **Binary:** TSVC benchmarks binaries for x86
  - **Run-time environment:** Ubuntu 18.04 with Docker installed
  - **Hardware:** Any x86 machine with 4 physical CPUs, 16 GiB of RAM, ~15 GiB disk space, Broadband connection
  - **Output:** A set of .csv files and the binary files with updated debugging headers
  - **Experiments:** Build the docker file, start a container and run the shell scripts
  - **How much disk space required (approximately)?:** 8 GiB
  - **How much time is needed to prepare workflow (approximately)?:** ~1 hr (depends on the internet speed and machine configuration)
  - **How much time is needed to complete experiments (approximately)?:** ~30 minutes

## Description

### How delivered

The source code, object files for benchmarks, dockerfile and shell scripts are available in this archive.

### Hardware dependencies

Any x86 machine with 4 physical CPUs, 16 GiB of RAM, ~15 GiB disk space and broadband connection should be fine.

### Software dependencies

Tested on Ubuntu 18.04 (x86_64), should work on similar Linux distributions.

## Installation

The artifact is packaged as a Docker application.  Installation of Docker is covered [here](https://docs.docker.com/engine/install/).  
Follow these steps for building and installing the tool :

1. [Install Docker Engine](https://docs.docker.com/engine/install/) and set it up. Make sure you are able to run the [hello-world example](https://docs.docker.com/get-started/#test-docker-installation).

2. Go to the top-level directory of the unpacked archive and build the Docker image. Note that internet connectivity is required in this step.
   ```
   docker build -t cgo22-debugheaders .
   ```
   This process can take a while depending upon your internet connection bandwidth.  
   Note: You may want to add `--build-arg=http{s}_proxy="http://proxy-url:proxy-port"` to the docker build command, if you are behind a proxy.

3. Run a container with the built image.
   ```
   docker run -it cgo22-debugheaders
   ```

4. (Inside the container) Build the artifact and install the tool.
   ```
   ./build.sh
   ```
   Note: If you're behind a proxy, you may want to set the proxy environment variables -- `http_proxy` and `https_proxy` before you run `build.sh`.

## Experiment workflow

Once the installation is done, run the experiments for the three compilers gcc, clang/llvm and icc using the below command:
```
./run-all.sh
```

The updated object files for the TSVC benchmarks will be stored inside the directory: `bin/rewrites_dir_all`.

## Evaluation and expected results

### Reproducing the results from tables 2 and 3

Both tables 2 and 3 from the paper have the same structure and contain the results for the compilers - clang, gcc and icc.  
These can be reproduced using the `build.sh` and `run-all.sh` scripts.
The results will be stored as a set of .csv files inside the `bin/results-{clang,gcc,icc}-all.csv` files.

The .csv files have same structure as the tables 2 and 3 including matching header names and presentation format.

### Reproducing the ablation studies results

The results for ablation studies can be reproduced in a similar manner as above -- after building the artifact with `build.sh`, the respective shell scripts
can be used for running each of the variants; the resulting CSV files would have same structure as described above.
The shell scripts and their respective variants are:
1. `run-all.sh` for forward + backward DFA with reversibility (table 2 and 3).  The output files would have `-all` suffix.
2. `run-rev-comp.sh` for only forward with reversibility.  The output files would have `-rev-comp` suffix.
3. `run-basic-dfa.sh` for only forward DFA without reversibility.  The output files would have `-basic-dfa` suffix.
4. `run-no-dfa.sh` for no DFA.  The output files would have `-no-dfa` suffix.

The results for the 1st setting (`-all`) should already be available in `results-{clang,gcc,icc}-all.csv` if you ran *build.sh* and *run-all.sh* as specified in previous section.
The results for the 2nd, 3rd and 4th setting for each compiler can be generated by running the appropriate script.

The structure of the resulting .csv files is same as described in previous section.
The updated object files for respective variants are stored in `bin/rewrites_dir_rev_comp`, `bin/rewrites_dir_basic_dfa`, and `bin/rewrites_dir_no_dfa` respectively.

The updated object files can be inspected with utilities such as `objdump` and `readelf`.
For comparison, the original object files are present inside `eqfiles/TSVC_combined` directory.

### Testing the updated object file using gdb

You can test running the updated object file (e.g. `s000` for clang) in gdb, as shown below:
```
cd /home/user/artifact/TSVC_source_files
# link the binary
clang-9 ../bin/rewrites_dir_all/s000.clang-rewrite.o ./tsvc.c -m32 -o s000
# now run gdb
gdb s000
(inside gdb) break s000
(inside gdb) run
```
Now you can step through gdb and print the values of variables:
```
(inside gdb) step
(inside gdb) print i
```

The expected output is shown below:
```
Starting program: /home/user/artifact/bin/archived-results/rewrites_dir_all/s000
warning: Error disabling address space randomization: Operation not permitted

Breakpoint 1, s000 ()
    at /home/shubhani/superopt-project-perf/superopt-tests/TSVC_prior_work/s000.c:9
9                       for (int i = 0; i < lll; i++) {
(inside gdb) step
10                              X[i] = Y[i] + val;
(inside gdb) print i
$1 = 0
(inside gdb) step
9                       for (int i = 0; i < lll; i++) {
(inside gdb) print i
$2 = 0
(inside gdb) step
10                              X[i] = Y[i] + val;
(inside gdb) print i
$3 = 16
(inside gdb) step
9                       for (int i = 0; i < lll; i++) {
(inside gdb) print i
$4 = 16
(inside gdb) step
10                              X[i] = Y[i] + val;
(inside gdb) print i
$5 = 32
(inside gdb) step
9                       for (int i = 0; i < lll; i++) {
(inside gdb) print i
$6 = 32
(inside gdb) step
10                              X[i] = Y[i] + val;
(inside gdb) print i
$7 = 48
(inside gdb) quit
```

Initially, when we run 'step' inside the gdb, `i = 0` is run and we get a value of 0. 
After that, when we run a sequence of gdb commands below,
```
step
print i
```
the two source statements - `X[i] = Y[i] + val;` and `i < lll; i++` run alternately
and we can see the value of source variable 'i' getting incremented by 16, each time 
the latter statement is run.

## Structure

* `eqfiles.tgz` contains the proof files as produced by an equivalence checker. These files will be used as input to the frontend of our tool.
* `bin.tgz` contains the frontend part of the tool along with the required libraries. This part of the tool processes the input from equivalence checker and generates postfix expressions to be passed on to the backend part of the tool.
* The `gimli_write` directory contains the source code for the backend part of the tool which does the processing and updates to the debug headers in the optimized object files.
* The `eval` directory has the code for automatic evaluator that compares the optimized object files before and after they are updated and produces the CSV results.
* The `TSVC_source_files` directory contains the source programs for TSVC benchmarks.
* `archived-results.tgz` contains sample output including modified TSVC binaries and CSV files.
