# release process

This describes:

- the manual steps to release a new version
- the explanation of how the Gitlab CI jobs producing build artifacts
  works, and how to reproduce the locally if needed

These instructions assume you replace `$VERSION` with the actual
version being released.

## source artifacts

outputs:
- git tag
- main git branch updated for further development
- source tarball

operations:
- create release pull request, and get merged
  - check with `cargo outdated` not to miss any outdated dependency
  - update version in `Cargo.toml`
  - set release date in `CHANGELOG.md`, add link to prospective URL for Gitlab release
  - run `cargo tree` (or any cargo command updating the version in `Cargo.lock`
  - `git commit Cargo.toml Cargo.lock -m "Release version $VERSION"`
- `git tag $VERSION -m $VERSION`, push
- create post-release pull request
  - update version in Cargo.toml to $NEXTVERSION-dev
  - create new entry for $NEXTVERSION in `CHANGELOG.md`

### source tarball

The tarball itself is created by the CI from the tagged version, using
`cargo package`.  The generated `.crate` file is indeed a standard
gzipped source tarball.

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

To build it locally using rootless `podman`:

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
```

> **Note**
>
> As you see, using a generic container and `podman` in `keep-id` node
> makes some things spooky, we will try to improve this in the future.


### Linux `x86_64` "2017" binary

> **Warning**
>
> Instructions not working yet, [clang 3.8 apparently misses a symbol
> added in 3.5](https://github.com/KyleMayes/clang-sys/issues/163)

These instructions describe building the binary in a `debian:9`
container environment, so it could run in all Linux distros released
since 2017.  Set it up with:

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

## binary packages

Those packaging instructions do not attempt to build from source, for
reasons [explained here](FIXME).  Instead they rely on the prebuilt
binaries being available in the same directory as the source tree
(accessible as `..` from within the source tree).  If you wish to
package locally-built binaries, adjust accordingly.

outputs:
- `xen-guest-agent-$VERSION-0.fc37.x86_64.rpm`
- `xen-guest-agent-debuginfo-$VERSION-0.fc37.x86_64.rpm`
- `xen-guest-agent_$VERSION_amd64.deb`
- `xen-guest-agent-dbgsym_$VERSION_amd64.deb`

### rpm packages

This is the basis of the `rpm-x86_64` CI job.

> **Note**
>
> RHEL and their rebuilds do not have `xen-libs` package readily
> available, so will need more work and is not covered here yet.

These instructions describe building the RPM in a container
environment such as `fedora:37`, using rootless podman.  Note that the
specfile, which contains the version and packaging date, has to be
generated first from the `.in` template:

```
xen-guest-agent$ sed -e "s/@@VERSION@@/$VERSION/" -e "s/@@UPSTREAMVERSION@@/$UPSTREAMVERSION/" -e "s/@@AUTHOR@@/$USER <$EMAIL>/" -e "s/@@DATE@@/$(date +"%a %b %d %Y")/" < xen-guest-agent.spec.in > xen-guest-agent.spec
xen-guest-agent$ mkdir -p SOURCES
xen-guest-agent$ ln -sr ../xen-guest-agent-$UPSTREAMVERSION-linux-x86_64 SOURCES/xen-guest-agent
xen-guest-agent$ ln -sr startup/xen-guest-agent.service SOURCES/
xen-guest-agent$ podman run -v $PWD/..:/data --userns=keep-id -u root -it --rm fedora:37 bash
[root /]# dnf install -y rpm-build dnf-utils
[root /]# dnf builddep /data/xen-guest-agent/xen-guest-agent.spec -y
```

Build the RPM:

```
[root /]# sudo -u user -i
$ cd /data/xen-guest-agent
$ rpmbuild -bb xen-guest-agent.spec --define "_topdir $(pwd)"
```

### deb package

This is the basis of the `deb-amd64` CI job.

We build in a Debian 10 container, using deb `debian/` directory from
the git tree, but the prebuilt binary (by default in `..`).  Note that
the `debian/changelog` file, which contains the version and packaging
date, has to be generated first from the `.in` template:

```
xen-guest-agent$ sed -e "s/@@VERSION@@/$VERSION/" -e "s/@@AUTHOR@@/$USER <$EMAIL>/" -e "s/@@DATE@@/$(date --rfc-822)/" < debian/changelog.in > debian/changelog
xen-guest-agent$ podman run -v $PWD/..:/data --userns=keep-id -u root -it --rm debian:10 bash
[root /]# apt update
[root /]# apt install -y build-essential debhelper
[root /]# su - user
$ cd /data/xen-guest-agent
$ dpkg-checkbuilddeps
$ fakeroot debian/rules binary
```
