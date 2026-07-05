# Phase 2 Architecture: The Bootable Distro

This document specifies the low-level lifecycle, filesystem mount dependencies, graphics compositor layers, and memory staging boundaries introduced in **Phase 2: The Distro** of the Octos operating system framework.

---

## 1. Custom Rust PID 1 Init Lifecycle
Traditional init subsystems (like Systemd or SysVInit) are completely omitted in favor of a specialized Rust binary (`crates/octos-init`) running directly as process identifier 1 (PID 1).

### VFS Mount Sequence
During boot execution, the init kernel module invokes libc system calls to establish virtual filesystem mount points:
1. **`/proc`** (`procfs`): Mapped for kernel/process boundary queries.
2. **`/sys`** (`sysfs`): Mapped for device configuration and hardware info.
3. **`/dev`** (`devtmpfs` with a backup fallback to `tmpfs`): Mapped to query active hardware nodes.

```text
[Hardware Bootstrap]
        │
        ▼
[Custom Rust Init (PID 1)]
        │
        ├──► 1. Mount virtual systems (/proc, /sys, /dev)
        │
        ├──► 2. Invoke Model Loader (stage weights to RAM)
        │
        └──► 3. Spawn & Supervise Core Daemon (octos-core)
```

---

## 2. Model Weight RAM Allocation Layout
Prior to launching user interface systems or compositor event loops, the bootloader stages LLM or ONNX weights (e.g. `llama-3b-q4.gguf`) directly into a fixed continuous block of physical memory to guarantee predictable sub-millisecond local inference.

- **Mock Alloc Address Range**: `[0x7FFF00000000 - 0x7FFF20000000]` for a 512 MB allocation.
- **Log Verifications**: The model memory footprint is tracked inside `model_loader.rs` structures, ensuring memory limits are respected.

---

## 3. Wayland Composer Event Loop & Compositor Server
The graphics server (`crates/octos-compositor`) is built on top of the **Smithay** compositor framework.
- **`OctosDisplayServer`**: Tracks screen geometries (`smithay::utils::Rectangle`), logical input scopes, and active client surfaces.
- **`render_agent_card`**: Translates intent actions (e.g. `approve_payment`) from incoming IAC network packets into visual RGBA frame canvas graphics buffers on display outputs.

---

## 4. Rootfs Directory Structure
The Alpine Linux root filesystem (rootfs) layout replicated by the builder contains:
- `/bin`: Core utilities (e.g., shells).
- `/sbin/init`: Symlink or location of our custom `octos-init` PID 1 binary.
- `/etc`: Configuration files.
- `/proc`, `/sys`, `/dev`: Mount targets.
- `/root`: root user home directories.
- `/usr/bin`: Additional system applications.
