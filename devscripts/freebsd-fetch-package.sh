#!/bin/sh
set -e

PKGNAME="$1"

BASE_URL="http://pkg.freebsd.org/FreeBSD:13:amd64/release_2"
PKG=$(curl -s ${BASE_URL}/packagesite.txz |
      unxz | tar -x packagesite.yaml -O |
      grep '"name":"'$PKGNAME'"' |
      sed 's/.*"path":"\([^"]*\)".*/\1/')
curl -O ${BASE_URL}/"$PKG"

echo "$(basename "$PKG")"
