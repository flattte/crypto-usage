#!/usr/bin/env bash

case $1 in
  "build")
    cargo build --release
    ;;
  "run")
    qemu-system-riscv64 -nographic -machine virt -bios none -kernel ./target/riscv64gc-unknown-none-elf/release/crypto-usage
    echo "qemu pid $!" 
  ;;
  "run-gdb")
    # -s option is a shorthand for -gdb tcp::1234 so the attach is on 1234 port
    # then `riscv64-unknown-elf-gdb exe.elf`
    # theni(gdb) target remote localhost:1234
    qemu-system-riscv64 -nographic -machine virt -bios none -kernel ./target/riscv64gc-unknown-none-elf/release/crypto-usage -s -S 
    echo "qemu pid $!" 
  ;;
  *)
    echo "cmds: firvcmd <run | build | run-gdb>"
  ;;
esac
