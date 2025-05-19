cd "$(dirname "$0")"
cd ".."

cargo build --target aarch64-unknown-none --release
