#!/usr/bin/bash

DEST=${1:-"ctr-bundle"}

mkdir -p "$DEST"/rootfs

# Download and untar an alpine image
curl -O https://dl-cdn.alpinelinux.org/alpine/v3.14/releases/x86_64/alpine-minirootfs-3.14.2-x86_64.tar.gz
tar xf alpine-minirootfs-3.14.2-x86_64.tar.gz -C "$DEST"/rootfs
rm alpine-minirootfs-3.14.2-x86_64.tar.gz

# Create an ubuntu based root filesystem
#podman export $(podman create ubuntu) | tar -C rootfs -xvf -

pushd "$DEST"

# Generate a runtime spec
runc spec --rootless

popd
