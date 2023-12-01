#!/bin/sh

set -e

if [ -z "${IN_DOCKER}" ]; then
    cargo clean
    docker run -u $(id -u):$(id -g) \
               -v /etc/passwd:/etc/passwd:ro \
               -v /etc/group:/etc/group:ro \
               -v $(pwd):$(pwd) \
               -w $(pwd) \
               -e IN_DOCKER=1 \
               --rm -it rust:latest ./build-deb.sh
else
    cargo install cargo-deb
    cargo deb
fi
