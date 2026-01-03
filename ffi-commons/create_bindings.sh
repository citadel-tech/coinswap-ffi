#!/bin/bash

OS_TYPE=$(uname -s)
case "$OS_TYPE" in
    Darwin*)
        LIB_EXTENSION="dylib"
        ;;
    Linux*)
        LIB_EXTENSION="so"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        LIB_EXTENSION="dll"
        LIB_PREFIX=""
        ;;
    *)
        echo "Unknown operating system: $OS_TYPE"
        echo "Defaulting to .so extension"
        LIB_EXTENSION="so"
        ;;
esac

if [[ "$OS_TYPE" == MINGW* ]] || [[ "$OS_TYPE" == MSYS* ]] || [[ "$OS_TYPE" == CYGWIN* ]]; then
    LIBRARY_PATH="./target/release/coinswap_ffi.$LIB_EXTENSION"
else
    LIBRARY_PATH="./target/release/libcoinswap_ffi.$LIB_EXTENSION"
fi

echo "Building release library..."
cargo build --release

if [ ! -f "$LIBRARY_PATH" ]; then
    echo "Error: Library not found at $LIBRARY_PATH"
    exit 1
fi

echo "Using library: $LIBRARY_PATH"

languages=("kotlin" "swift" "python" "ruby")

for lang in "${languages[@]}"; do
    echo "Generating $lang bindings..."
    cargo run --bin uniffi-bindgen generate \
        --library "$LIBRARY_PATH" \
        --language "$lang" \
        --out-dir "./bindings/$lang" \
        --no-format
    
    if [ $? -eq 0 ]; then
        echo "✓ $lang bindings generated successfully"
    else
        echo "✗ Failed to generate $lang bindings"
        exit 1
    fi
done

echo ""
echo "All bindings generated successfully!"