FROM ubuntu:18.04

ARG DEBIAN_FRONTEND=noninteractive
# enable i386 packages support
RUN dpkg --add-architecture i386
# install build-essential and other required dependencies
RUN apt-get update && apt-get install -y curl clang-9 gdb build-essential libgmp-dev libboost-all-dev libc6-dev-i386 gcc-8-multilib
# add non-root user
RUN groupadd -r user && \
    useradd -r -g user -d /home/user -s /bin/bash -c "Docker image user" user && \
    mkdir -p /home/user && \
    chown -R user:user /home/user
# copy everything to user directory
COPY --chown=user . /home/user/artifact/
WORKDIR /home/user/artifact
# switch to non-root user
USER user
ENV LD_LIBRARY_PATH      /home/user/artifact/bin
ENV LOGNAME user
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN echo 'set substitute-path /home/shubhani/superopt-project-perf/superopt-tests/TSVC_prior_work/ /home/user/artifact/TSVC_source_files/' >> /home/user/.gdbinit
