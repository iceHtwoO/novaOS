#!/bin/bash

cd "$(dirname "$0")"

set -a
source ../.env
set +a

set -e  # Stop on errors


# === RESOLVE VARIABLES ===
REMOTE="$REMOTE_USER@$REMOTE_HOST"
REMOTE_DIR="$TFTP_PATH"

# === BUILD ===
echo "[*] Building kernel..."
eval $BUILD_COMMAND

# === CONVERT TO IMG ===
echo "[*] Convert kernel elf to img..."
llvm-objcopy -O binary "../$BUILD_PATH/$BINARY_NAME" ../$BUILD_PATH/kernel8.img


# === COPY TO TFTP ===
echo "[*] Copying firmware files to TFTP server..."
scp ../firmware_files/* "$REMOTE:$REMOTE_DIR/."
echo "[*] Copying kernel to TFTP server..."
scp "../$BUILD_PATH/kernel8.img" "$REMOTE:$REMOTE_DIR/$KERNEL_NAME"

echo "[✓] Deployed to TFTP server as $KERNEL_NAME"
