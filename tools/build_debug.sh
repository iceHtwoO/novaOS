cd "$(dirname "$0")"
cd ".."

cargo build --target aarch64-unknown-none
llvm-objcopy -O binary ../target/aarch64-unknown-none/debug/nova ../target/aarch64-unknown-none/debug/kernel8.img
