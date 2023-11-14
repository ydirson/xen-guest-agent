# release process

This describes the current state of things, most of this will get
automated by scripts and CI jobs.

These instructions assume you replace `$VERSION` with the actual
version being released.

## source artfacts

outputs:
- git tag
- main git branch updated for further development
- source tarball

operations:
- update version in Cargo.toml, xen-guest-agent.spec, debian/changelog
- `git tag $VERSION -m $VERSION`
- `cargo package`
- `mv target/package/xen-guest-agent-$VERSION.crate ../xen-guest-agent-$VERSION.tar.gz`
- update version in Cargo.toml to $NEXTVERSION-dev

## binary artifacts

outputs:
- `xen-guest-agent-$VERSION-linux-x86_64`
- `xen-guest-agent-$VERSION-freebsd-x86_64`

> **Note**
>
> The following instructions install the latest Rust toolchain, you
> may want to adjust them for full reproducibility.

### Linux `x86_64` "2019" binary

These instructions describe building the binary in a `debian:10`
container environment, so it could run in all Linux distros released
since 2019.  They are the basis of the `build-release-linux-x86_64` CI
job.

To build it locally using `podman`:

```
xen-guest-agent$ podman run -v $PWD/..:/data --userns=keep-id -u root -it --rm debian:10 bash
[root /]# apt update
[root /]# apt install -y curl
[root /]# apt install -y build-essential libxen-dev clang
[root /]# install -d -m 755 -o user /.cargo /.rustup
[root /]# su - user
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
$ . "/.cargo/env"
$ cd /data/xen-guest-agent
$ cargo clean
$ cargo build --release
$ mv target/release/xen-guest-agent ../xen-guest-agent-$VERSION-linux-x86_64
$ exit
[root /]# exit
```

> **Note**
>
> As you see, using a generic container and `podman` in `keep-id` node
> makes some things spooky, we will try to improve this in the future.


### Linux `x86_64` "2017" binary

> **Warning**
>
> Instructions not working yet, clang 3.8 too old despite bindgen
> claiming it needs only 3.5?

These instructions describe building the binary in a `debian:9` Docker
environment, so it could run in all Linux distros released since 2017.
Set it up with:

```
xen-guest-agent$ docker run -v $PWD/..:/data -it --rm debian:9 bash
[root xen-guest-agent]# sed -i s,http://deb.debian.org/debian,http://archive.debian.org/debian-archive/debian, /etc/apt/sources.list
[root xen-guest-agent]# apt update
[root xen-guest-agent]# apt install -y curl --allow-downgrades libnettle6/oldoldstable
[root xen-guest-agent]# curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

(just hit ENTER to *Proceed with installation*)

Then build:

```
[root /]# cd /data/xen-guest-agent
[root xen-guest-agent]# apt install -y build-essential libxen-dev clang
[root xen-guest-agent]# . "$HOME/.cargo/env"
[root xen-guest-agent]# cargo clean
[root xen-guest-agent]# cargo build --release
[root xen-guest-agent]# mv target/release/xen-guest-agent ../xen-guest-agent-$VERSION-linux2017-x86_64
```

### FreeBSD `x86_64` binary

> **Note**
>
> These instructions do not imply using a clean container (yet); if
> you expect reproducibility some extra measures might be needed.

On old-enough FreeBSD x86_64 guest version (FreeBSD 12):

```
BINDGEN_EXTRA_CLANG_ARGS=-I/usr/local/include \
RUSTFLAGS=-L/usr/local/lib \
cargo build --release --no-default-features -F xenstore,net_pnet

mv target/release/xen-guest-agent ../xen-guest-agent-$VERSION-freebsd-x86_64
```
