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

# Define output directories for each language
KOTLIN_DIR="../coinswap-kotlin"
SWIFT_DIR="../coinswap-swift"
PYTHON_DIR="../coinswap-python"
RUBY_DIR="../coinswap-ruby"

# Create directories if they don't exist
mkdir -p "$KOTLIN_DIR"
mkdir -p "$SWIFT_DIR"
mkdir -p "$PYTHON_DIR"
mkdir -p "$RUBY_DIR"

echo ""
echo "Generating Kotlin bindings..."
cargo run --bin uniffi-bindgen generate \
    --library "$LIBRARY_PATH" \
    --language "kotlin" \
    --out-dir "$KOTLIN_DIR" \
    --no-format

if [ $? -eq 0 ]; then
    echo "✓ Kotlin bindings generated at $KOTLIN_DIR"
else
    echo "✗ Failed to generate Kotlin bindings"
    exit 1
fi

echo ""
echo "Generating Swift bindings..."
cargo run --bin uniffi-bindgen generate \
    --library "$LIBRARY_PATH" \
    --language "swift" \
    --out-dir "$SWIFT_DIR" \
    --no-format

if [ $? -eq 0 ]; then
    echo "✓ Swift bindings generated at $SWIFT_DIR"
else
    echo "✗ Failed to generate Swift bindings"
    exit 1
fi

echo ""
echo "Generating Python bindings..."
cargo run --bin uniffi-bindgen generate \
    --library "$LIBRARY_PATH" \
    --language "python" \
    --out-dir "$PYTHON_DIR" \
    --no-format

if [ $? -eq 0 ]; then
    echo "✓ Python bindings generated at $PYTHON_DIR"
else
    echo "✗ Failed to generate Python bindings"
    exit 1
fi

echo ""
echo "Generating Ruby bindings..."
cargo run --bin uniffi-bindgen generate \
    --library "$LIBRARY_PATH" \
    --language "ruby" \
    --out-dir "$RUBY_DIR" \
    --no-format

if [ $? -eq 0 ]; then
    echo "✓ Ruby bindings generated at $RUBY_DIR"
else
    echo "✗ Failed to generate Ruby bindings"
    exit 1
fi

echo ""
echo "Copying library files to language directories..."

for DIR in "$KOTLIN_DIR" "$SWIFT_DIR" "$PYTHON_DIR" "$RUBY_DIR"; do
    echo "Copying files to $DIR..."
    
    # Copy the main library
    cp "$LIBRARY_PATH" "$DIR/"
    
    if [[ "$OS_TYPE" == MINGW* ]] || [[ "$OS_TYPE" == MSYS* ]] || [[ "$OS_TYPE" == CYGWIN* ]]; then
        cp "./target/release/coinswap_ffi.d" "$DIR/" 2>/dev/null || echo "  ⚠ coinswap_ffi.d not found (optional)"
    else
        cp "./target/release/libcoinswap_ffi.d" "$DIR/" 2>/dev/null || echo "  ⚠ libcoinswap_ffi.d not found (optional)"
    fi
    
    if [[ "$OS_TYPE" == MINGW* ]] || [[ "$OS_TYPE" == MSYS* ]] || [[ "$OS_TYPE" == CYGWIN* ]]; then
        cp "./target/release/uniffi-bindgen.exe" "$DIR/" 2>/dev/null || echo "  ⚠ uniffi-bindgen.exe not found (optional)"
    else
        cp "./target/release/uniffi-bindgen" "$DIR/" 2>/dev/null || echo "  ⚠ uniffi-bindgen not found (optional)"
    fi
    
    cp "./target/release/uniffi-bindgen.d" "$DIR/" 2>/dev/null || echo "  ⚠ uniffi-bindgen.d not found (optional)"
    
    echo "  ✓ Files copied to $DIR"
done

echo ""
echo "All bindings generated successfully!"
echo ""
echo "Generated bindings:"
echo "  Kotlin:  $KOTLIN_DIR"
echo "  Swift:   $SWIFT_DIR"
echo "  Python:  $PYTHON_DIR"
echo "  Ruby:    $RUBY_DIR"
echo ""
echo "Copied files to each directory:"
echo "  - libcoinswap_ffi.$LIB_EXTENSION (main library)"
echo "  - libcoinswap_ffi.d (debug symbols)"
echo "  - uniffi-bindgen (binary)"
echo "  - uniffi-bindgen.d (debug symbols)"
echo ""
echo "See language-specific README files for usage:"
echo "  - ../coinswap-kotlin/README.md"
echo "  - ../coinswap-swift/README.md"
echo "  - ../coinswap-python/README.md"
echo "  - ../coinswap-ruby/README.md"