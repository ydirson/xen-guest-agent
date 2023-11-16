# Cross building the tools

The [Rust cross tool](https://github.com/cross-rs/cross) provides
containers ready with toolchain and basic sysroot.

Compiled binaries land under `target/x86_64-unknown-freebsd/`.

## cross-building to FreeBSD locally

The `cross` tool works out of the box as long as no extra library is
needed, which means it requires extra work for libxenstore.

E.g. this works:
```
cross build --target x86_64-unknown-freebsd --no-default-features -F net_pnet
```

Building against native libs requires a more recent `cross` version
than the last published 0.2.5 (where container image for FreeBSD build
has a version of `libclang` too old for recent `bindgen`, whereas old
`bindgen` had broken cross-build support).  It must thus be installed
for now from git repo:

```
cargo install --git https://github.com/cross-rs/cross cross
```

libxenstore is provided by the `xen-tools` port/package.  The
container provides support for installing packages, though
undocumented and still [with
problems](https://github.com/cross-rs/cross/issues/1367).

The sysroot is at `/usr/local/x86_64-unknown-freebsd12/` in the
container, and currently has a problem, as libs are installed under
this in `include/` and `lib/`, whereas the toolchain wants them in
`usr/include/` and `usr/lib/`.  So we start by creating symlinks so
the toolchain will find everything.

The pkg file is a tarball containing files under `/usr/local/`, we
have to replace this with the proper prefix within the sysroot.

All those steps can be described as a `pre-build` hook in `Cross.toml`.

Use with simply with:
```
cross build --target x86_64-unknown-freebsd --no-default-features -F net_pnet,xenstore
```

## cross-building to FreeBSD in Gitlab CI

In Gitlab CI we use Docker runners.  It makes little sense to use
`cross` inside a Docker container with the Rust toolchain, just to
have it launch a nested container with the cross environment (while
sharing the Rust toolchain into the cross container).  Especially as
there are as yet unresolved
[issues](https://github.com/cross-rs/cross/issues/1351) with running
`cross` nested.

It is however useful to use the `cross-rs` container directly in
Gitlab CI.  It does not have Rust preinstalled however (the `cross`
binds it from the host), so we must take care of it.
