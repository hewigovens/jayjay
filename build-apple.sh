#!/bin/bash
set -euo pipefail

CRATE_NAME="jayjay_uniffi"
LIB_NAME="lib${CRATE_NAME}.a"
BINDINGS_DIR="./build/bindings"
HEADERS_DIR="${BINDINGS_DIR}/Headers"
XCFRAMEWORK_OUTPUT="./shell/mac/JayJayFFI.xcframework"
SWIFT_OUT="./shell/mac/Sources/JayJayBindings"
TARGET="aarch64-apple-darwin"

echo "==> Building Rust (${TARGET})..."
cargo build --release --target "${TARGET}" -p jayjay-uniffi

echo "==> Generating Swift bindings..."
mkdir -p "${BINDINGS_DIR}"
cargo run --release -p jayjay-uniffi --bin uniffi-bindgen generate \
    --library "target/${TARGET}/release/${LIB_NAME}" \
    --language swift \
    --out-dir "${BINDINGS_DIR}"

echo "==> Preparing headers..."
mkdir -p "${HEADERS_DIR}"
cp "${BINDINGS_DIR}/${CRATE_NAME}FFI.h" "${HEADERS_DIR}/"
cp "${BINDINGS_DIR}/${CRATE_NAME}FFI.modulemap" "${HEADERS_DIR}/module.modulemap"

echo "==> Copying Swift sources..."
mkdir -p "${SWIFT_OUT}"
cp "${BINDINGS_DIR}/${CRATE_NAME}.swift" "${SWIFT_OUT}/"

echo "==> Creating XCFramework..."
rm -rf "${XCFRAMEWORK_OUTPUT}"
xcodebuild -create-xcframework \
    -library "target/${TARGET}/release/${LIB_NAME}" \
    -headers "${HEADERS_DIR}" \
    -output "${XCFRAMEWORK_OUTPUT}"

echo "==> Done! XCFramework at ${XCFRAMEWORK_OUTPUT}"
echo "    Swift bindings at ${SWIFT_OUT}/${CRATE_NAME}.swift"
