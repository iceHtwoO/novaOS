#!/bin/bash
set -e

# Config
IMAGE_NAME="sd.img"
IMAGE_SIZE_MB=64
FIRMWARE_DIR="./firmware_files"


# Clean up existing image
if [ -f "$IMAGE_NAME" ]; then
    echo "[*] Removing existing $IMAGE_NAME..."
    rm -f "$IMAGE_NAME"
fi

# Create empty image
echo "[*] Creating ${IMAGE_SIZE_MB}MB SD image..."
dd if=/dev/zero of="$IMAGE_NAME" bs=1M count=$IMAGE_SIZE_MB

# Format image as FAT32
echo "[*] Formatting image as FAT32..."
mformat -i sd.img -F ::

# Copy all files from firmware_files/ into root of SD image
echo "[*] Copying files from '$FIRMWARE_DIR' to image..."
mcopy -i "$IMAGE_NAME" -s "$FIRMWARE_DIR"/* ::/

echo "[✓] SD card image '$IMAGE_NAME' is ready."
