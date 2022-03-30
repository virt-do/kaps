# Kaps

<img src="https://img.shields.io/github/workflow/status/virt-do/kaps/run0%20build%20and%20unit%20tests?style=for-the-badge" />

`kaps` is an experimental OCI container runtime written in Rust. The project aims to provide a performant, intuitive and OCI-compliant CLI allowing users to run & manage containers. 

**Project is experimental and should not be used in any production systems.**

## Install

To easily install `kaps` on your computer, simply run : 

```shell
$ cargo install --path . && sudo mv $HOME/.cargo/bin/kaps /usr/local/bin
```

## Quickstart

Here a little quickstart to run an `alpine` container, depending on your architecture, for `amd64` : 

**At the moment, Kaps need to be run with root privileges. Consider adding `sudo` before each command or directly execute the following commands as root.**

```shell
# --name is used to give an identifier to our image. See reference for more information.
$ kaps pull docker.io/amd64/alpine --name alpine
# Mount the image as an OCI bundle into /tmp/alpine
$ kaps mount alpine /tmp/alpine
# Run your container with the bundle
$ kaps run --bundle /tmp/alpine
```

For more documentation about commands, please see : 
[Command line reference](docs/cli-reference.md)