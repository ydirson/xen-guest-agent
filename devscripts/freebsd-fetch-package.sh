#!/bin/sh
set -e

PKGNAME="$1"

PKG=$(curl -s http://pkg.freebsd.org/FreeBSD:12:amd64/release_4/packagesite.txz |
      unxz | tar -x packagesite.yaml -O |
      grep '"name":"'$PKGNAME'"' |
      sed 's/.*"path":"\([^"]*\)".*/\1/')
curl -O http://pkg.freebsd.org/FreeBSD:12:amd64/release_4/"$PKG"

echo "$(basename "$PKG")"
