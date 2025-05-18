cd "$(dirname "$0")"

qemu-system-aarch64 -M raspi3b -cpu cortex-a53 -serial stdio -display none -kernel ../target/aarch64-unknown-none/debug/nova -s
