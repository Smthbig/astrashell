# AstraShell

**Next-generation rootless Android Linux runtime**

WSL + Docker Desktop + lightweight Linux distro + Android-native integration — on your phone, without root.

## Why AstraShell?

| Feature | Termux | PRoot | Podroid | Droidspaces | **AstraShell** |
|---------|--------|-------|---------|-------------|----------------|
| No root required | ✅ | ✅ | ✅ | ❌ | ✅ |
| Real Linux kernel | ❌ | ❌ | ✅ (VM) | ✅ (shared) | ✅ (hybrid) |
| systemd support | ❌ | ❌ | ✅ | ✅ | ✅ |
| Full package compat | Partial | Slow VM | VM only | ✅ | ✅ |
| Near-native speed | ✅ | ❌ (~1000x) | VM overhead | ✅ | ✅ |
| GPU acceleration | ❌ | ❌ | ❌ | Partial | ✅ (Phase 3) |
| Docker/Podman | ❌ | ❌ | ✅ | ✅ | ✅ |
| AUR support | ❌ | Slow | ❌ | Partial | ✅ |
| Secure isolation | ❌ | Weak | VM | Namespace | Hybrid |
| Battery aware | ❌ | ❌ | ❌ | ❌ | ✅ |

## Architecture

AstraShell uses a **hybrid execution engine** that auto-selects the best mode:

```
┌────────────┐  ┌──────────────┐  ┌───────────┐
│  NATIVE    │  │  CONTAINER   │  │  MICROVM  │
│  (fast)    │  │  (isolated)  │  │  (secure) │
├────────────┤  ├──────────────┤  ├───────────┤
│ CLI tools  │  │ systemd      │  │ Full dist │
│ compilers  │  │ Docker       │  │ GUI apps  │
│ scripts    │  │ services     │  │ untrusted │
│ node/vim   │  │ databases    │  │ code      │
└────────────┘  └──────────────┘  └───────────┘
```

- **Native**: LD_PRELOAD + seccomp user notification — zero overhead for everyday tools
- **Container**: Full Linux namespaces (user, pid, mount, net, uts, ipc) — complete isolation
- **MicroVM**: AVF-backed crosvm/KVM — hardware-virtualized security (Phase 3)

## Quick Start

```bash
# Clone
git clone https://github.com/astrashell/astrashell
cd astrashell

# Build runtime
./build/build-rust.sh release

# Build Android app
cd android-app && ./gradlew assembleDebug

# Install APK on device
adb install app/build/outputs/apk/debug/app-debug.apk

# Push Arch Linux rootfs
adb push astrashell-arch-rootfs.tar.gz /sdcard/
```

Or use the pre-built APK from Releases.

## Requirements

- Android 13+ (API 33+)
- ARM64 device
- 2GB+ RAM (4GB recommended)
- 4GB+ free storage
- No root required

### Optional (for enhanced modes)
- AVF-supported device (Pixel 7+, etc.) for MicroVM mode
- Shizuku for AVF permission grant

## Project Structure

```
astrashell/
├── runtime/              # Core runtime (Rust)
│   ├── src/
│   │   ├── engine/       # Execution engines
│   │   ├── syscall/      # Syscall interception
│   │   ├── namespace/    # Linux namespace mgmt
│   │   ├── container/    # OCI container support
│   │   ├── vm/           # AVF microVM integration
│   │   ├── fs/           # Filesystem (overlay, FUSE)
│   │   ├── net/          # Networking (slirp, vsock)
│   │   ├── gui/          # Display (Wayland, X11)
│   │   ├── ipc/          # IPC bus, protocol
│   │   ├── security/     # Seccomp, capabilities
│   │   ├── util/         # Kernel detection, ELF
│   │   └── platform/     # Android-specific code
│   ├── c-api/            # C JNI interface
│   └── Cargo.toml
├── android-app/          # Android app (Kotlin)
│   ├── app/src/main/
│   │   ├── java/         # Jetpack Compose UI
│   │   ├── cpp/          # JNI bridge (C)
│   │   └── res/          # Resources
│   └── build.gradle.kts
├── config/               # Configuration files
├── build/                # Build scripts
├── docs/                 # Documentation
└── images/               # Distro rootfs scripts
```

## Build from Source

### Prerequisites

- Rust 1.75+ with `aarch64-linux-android` target
- Android NDK 26+
- JDK 17
- Android SDK 35

### Build

```bash
# Full build
./build/build-all.sh

# Or step by step
./build/build-rust.sh release
cd android-app && ./gradlew assembleDebug
```

## Configuration

Edit `/etc/astrashell/config.toml` in the container, or use the Settings screen in the app:

```toml
[runtime]
rootfs = "/data/data/com.astrashell/files/rootfs"
hostname = "astrashell"

[engine]
mode = "auto"  # native | container | vm | auto

[network]
type = "slirp"
port_forwards = ["2222:22", "8080:80"]

[gui]
enabled = false
display = "web"  # wayland | x11 | web
```

## Roadmap

- **Phase 1** (MVP): Basic runtime, terminal, namespace isolation, Arch rootfs
- **Phase 2**: Package managers, service management, OCI/Docker support
- **Phase 3**: AVF microVM, GPU acceleration, Wayland bridge
- **Phase 4**: Performance optimization, JIT syscall translation
- **Phase 5**: Developer ecosystem, cloud sync, IDE integration

## License

Apache 2.0

## Acknowledgments

- Android Virtualization Framework (AVF) team
- crosvm / Chromium OS
- Termux project
- Droidspaces project
- Podroid project
- NixOS AVF
- Arch Linux ARM
- gVisor / Kata Containers
- FEX-Emu / Box64
- Bubblewrap / nsjail
