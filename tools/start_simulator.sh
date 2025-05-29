cargo build --target aarch64-unknown-none --release
cd "$(dirname "$0")"

llvm-objcopy -O binary ../target/aarch64-unknown-none/release/nova ../target/aarch64-unknown-none/release/kernel8.img

qemu-system-aarch64 \
  -M raspi3b \
  -cpu cortex-a53 \
  -serial stdio \
  -sd ../sd.img \
  -display none \
  -kernel ../target/aarch64-unknown-none/release/kernel8.img
