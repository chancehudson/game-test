#!/bin/sh

set -e

cargo build --target wasm32-unknown-unknown --bin=client --release
wasm-bindgen --out-dir web --web target/wasm32-unknown-unknown/release/client.wasm

BUCKET_NAME="game"

wrangler r2 object put $BUCKET_NAME/index.html --file=web/index.html --remote
wrangler r2 object put $BUCKET_NAME/client.js --file=web/client.js --remote --content-type="application/javascript"
wrangler r2 object put $BUCKET_NAME/client_bg.wasm --file=web/client_bg.wasm --remote

# Function to get content type based on file extension
get_content_type() {
    local file="$1"
    local ext="${file##*.}"
    ext=$(echo "$ext" | tr '[:upper:]' '[:lower:]')

    case "$ext" in
        html|htm) echo "text/html" ;;
        js) echo "application/javascript" ;;
        wasm) echo "application/wasm" ;;
        css) echo "text/css" ;;
        json) echo "application/json" ;;
        png) echo "image/png" ;;
        jpg|jpeg) echo "image/jpeg" ;;
        gif) echo "image/gif" ;;
        svg) echo "image/svg+xml" ;;
        webp) echo "image/webp" ;;
        ico) echo "image/x-icon" ;;
        pdf) echo "application/pdf" ;;
        txt) echo "text/plain" ;;
        xml) echo "application/xml" ;;
        zip) echo "application/zip" ;;
        gz) echo "application/gzip" ;;
        tar) echo "application/x-tar" ;;
        mp3) echo "audio/mpeg" ;;
        mp4) echo "video/mp4" ;;
        webm) echo "video/webm" ;;
        ttf) echo "font/ttf" ;;
        woff) echo "font/woff" ;;
        woff2) echo "font/woff2" ;;
        *) echo "application/octet-stream" ;;
    esac
}

ASSETS_DIR="./assets"

find "$ASSETS_DIR" -type f ! -path "*/.*" ! -name ".*" | while read -r file; do
    # Get relative path (remove the source directory prefix)
    relative_path="${file#./}"

    # Get content type
    content_type=$(get_content_type "$file")

    # Get file size for display
    file_size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null || echo "unknown")

    echo "Uploading: $relative_path ($file_size bytes, $content_type)"

    # Upload the file
    if wrangler r2 object put "$BUCKET_NAME/$relative_path" \
        --file="$file" \
        --content-type="$content_type" \
        --remote \
        2>/dev/null; then
        echo "  ✓ Success"
        ((uploaded_count++))
    else
        echo "  ✗ Failed to upload $relative_path"
        ((error_count++))
    fi
done
