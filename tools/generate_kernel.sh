cd "$(dirname "$0")"

llvm-objcopy -O binary ../target/aarch64-unknown-none/release/nova ../kernel8.img
