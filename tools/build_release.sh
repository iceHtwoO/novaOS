cd "$(dirname "$0")"

cargo build --target aarch64-unknown-none --release
llvm-objcopy -O binary ../target/aarch64-unknown-none/release/nova ../target/aarch64-unknown-none/release/kernel8.img
