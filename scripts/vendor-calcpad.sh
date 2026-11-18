#!/usr/bin/env bash
set -euo pipefail

readonly UPSTREAM_REV="a348ab70bc2354386cb1e9f4a80246425f8304f0"
readonly UPSTREAM_REPO="https://github.com/civilkit/civilkit-calcs"
readonly ROOT="$(git rev-parse --show-toplevel)"
readonly VENDOR_DIR="${ROOT}/vendor/civilkit-calcs"
readonly TEMP_DIR="$(mktemp -d)"

cleanup() {
  rm -rf "${TEMP_DIR}"
}
trap cleanup EXIT

if [[ -e "${VENDOR_DIR}/calcpad-core" ]]; then
  echo "Vendor source already exists: ${VENDOR_DIR}/calcpad-core" >&2
  echo "Review it and remove it manually before repeating this pinned import." >&2
  exit 1
fi

mkdir -p "${VENDOR_DIR}"
curl --fail --location --silent --show-error --retry 3 \
  "${UPSTREAM_REPO}/archive/${UPSTREAM_REV}.tar.gz" \
  --output "${TEMP_DIR}/civilkit-calcs.tar.gz"

mkdir -p "${TEMP_DIR}/source"
tar -xzf "${TEMP_DIR}/civilkit-calcs.tar.gz" \
  --strip-components=1 \
  -C "${TEMP_DIR}/source"

test -f "${TEMP_DIR}/source/calcpad-core/Cargo.toml"
test -f "${TEMP_DIR}/source/LICENSE"
test -f "${TEMP_DIR}/source/NOTICE"

cp -a "${TEMP_DIR}/source/calcpad-core" "${VENDOR_DIR}/calcpad-core"
cp "${TEMP_DIR}/source/LICENSE" "${VENDOR_DIR}/LICENSE"
cp "${TEMP_DIR}/source/NOTICE" "${VENDOR_DIR}/NOTICE"
cp "${TEMP_DIR}/source/DEVIATIONS.md" "${VENDOR_DIR}/UPSTREAM_DEVIATIONS.md"
cp "${TEMP_DIR}/source/PORT_LOG.md" "${VENDOR_DIR}/UPSTREAM_PORT_LOG.md"

cat > "${VENDOR_DIR}/UPSTREAM.md" <<EOF
# CivilKit Calcpad source provenance

- Repository: ${UPSTREAM_REPO}
- Imported revision: ${UPSTREAM_REV}
- Imported component: calcpad-core
- Upstream package: civilkit-calcs-engine
- Rust library: calcpad_core
- Original engine lineage: CalcpadCE, MIT licensed.

The source is vendored in this repository. It is not a Git submodule or runtime Git dependency.
Upstream LICENSE and NOTICE are retained alongside the source. See the repository-root
DEVIATIONS.md for every Jabr-specific patch to vendored source.
EOF

echo "Imported CivilKit calcpad-core at ${UPSTREAM_REV}."
