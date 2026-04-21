#!/bin/bash
set -euo pipefail

# Build an SRPM for rpm-builder using the crate tarball from crates.io
# and a vendored dependency tarball.
#
# Usage:
#   ./build-srpm.sh              # uses version from Cargo.toml
#   ./build-srpm.sh 0.4.0        # explicit version
#
# Prerequisites: cargo, curl, rpmbuild
#
# The SRPM is written to the current directory.

VERSION="${1:-$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')}"
NAME="rpm-builder"
SOURCE_URL="https://crates.io/api/v1/crates/${NAME}/${VERSION}/download"
CRATE_FILE="${NAME}-${VERSION}.crate"
VENDOR_FILE="${NAME}-${VERSION}-vendor.tar.gz"

echo "Building SRPM for ${NAME}-${VERSION}"

BUILDDIR=$(mktemp -d)
trap 'rm -rf "$BUILDDIR"' EXIT

mkdir -p "$BUILDDIR"/{SOURCES,SPECS}

# Download the crate from crates.io
echo "Downloading ${CRATE_FILE}..."
curl -sL "${SOURCE_URL}" -o "$BUILDDIR/SOURCES/${CRATE_FILE}"

# Generate vendor tarball from the downloaded crate
echo "Vendoring dependencies..."
VENDORDIR=$(mktemp -d)
trap 'rm -rf "$BUILDDIR" "$VENDORDIR"' EXIT

tar -xf "$BUILDDIR/SOURCES/${CRATE_FILE}" -C "$VENDORDIR"
(cd "$VENDORDIR/${NAME}-${VERSION}" && cargo vendor --versioned-dirs > /dev/null)
tar -czf "$BUILDDIR/SOURCES/${VENDOR_FILE}" -C "$VENDORDIR/${NAME}-${VERSION}" vendor/

# Copy the spec file
cp rpm-builder.spec "$BUILDDIR/SPECS/"

# Build the SRPM
echo "Building SRPM..."
rpmbuild -bs \
    --define "_topdir $BUILDDIR" \
    --define "_srcrpmdir $(pwd)" \
    "$BUILDDIR/SPECS/rpm-builder.spec"

echo "Done. SRPM written to $(ls -1t "${NAME}"-*.src.rpm 2>/dev/null | head -1)"
