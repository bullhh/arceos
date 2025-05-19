# Platform Dynamic

This platform is a dynamic platform, which means that the platform can parse configuration dynamically during booting.

support platform:
    - aarch64 (fdt)

## How to run

### Qemu

Add feature `plat-dyn` witch will enable dynamic platform.

example:

```shell
make A=examples/helloworld ARCH=aarch64 SMP=2  LOG=trace FEATURES=plat-dyn,irq run
```

### Real board with uboot

First, we need `ostool` to build and upload the image to the board. It also supports windows.

```bash
cargo install ostool
```

 1. connect the board to the computer with serial port.

 2. if uboot has net:

    connect netwire to the board. The host pc and the board should be in the same network.

 3. run  `ostool run uboot`.
