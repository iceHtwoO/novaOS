# NovaOS

![NovaOS banner](docs/banner.png)

NovaOS is a hobby operating system kernel written in Rust for the Raspberry Pi 3 B+.
It is built as a learning project for low-level systems programming, bare-metal boot flow, and kernel development.

## At A Glance

NovaOS currently includes:

- UART initialization and logging
- Delay and sleep primitives
- Exception level transitions across EL2, EL1, and EL0
- GPIO control and interrupt handling
- Peripheral mailbox communication
- Framebuffer drawing primitives
- Heap memory allocation
- MMU initialization and translation table setup

Work in progress:

- SVC instruction handling
- Basic UART console improvements
- Multi-application management

Planned next:

- Multi-core support
- Dynamic clock speed management
- Kernel-independent applications
- Multiprocessing improvements

## Project Structure

- `src/` - kernel source, architecture code, peripherals, interrupts, and runtime
- `workspace/` - supporting crates such as `heap` and `nova_error`
- `tools/` - build, simulation, SD image generation, and deployment scripts
- `firmware_files/` - Raspberry Pi firmware files copied to SD or TFTP
- `link.ld` - linker script for the kernel image

## Requirements

You will need:

- Rust nightly toolchain (`rust-toolchain.toml` pins `nightly`)
- Rust target `aarch64-unknown-none`
- `llvm-objcopy` for generating `kernel8.img`
- `qemu-system-aarch64` for emulation
- `mtools` (`mformat`, `mcopy`) for SD image generation

Install the Rust target if needed:

```bash
rustup target add aarch64-unknown-none
```

## Build

Debug image:

```bash
cd tools
./build_debug.sh
```

Release image:

```bash
cd tools
./build_release.sh
```

Both scripts produce a `kernel8.img` under `target/aarch64-unknown-none/<profile>/`.

## Run In QEMU

1. Generate an SD image with firmware files:

```bash
cd tools
./generate_sd_card.sh
```

2. Start the emulator:

```bash
cd tools
./start_simulator.sh
```

For debug mode with the GDB stub enabled (`-S -s`):

```bash
cd tools
./start_simulator_debug.sh
```

## Deploy To Hardware

Use the TFTP workflow to deploy to a Raspberry Pi:

1. Copy `.env.example` to `.env` and fill in your values:
   - `REMOTE_USER`
   - `REMOTE_HOST`
   - `TFTP_PATH`
   - `BUILD_PATH`
   - `BINARY_NAME`
   - `KERNEL_NAME`
2. Run:

```bash
cd tools
./deply_to_hw.sh
```

## Notes

- This is an educational kernel project and is actively evolving.
- Interfaces and boot flow may change as features are added.
