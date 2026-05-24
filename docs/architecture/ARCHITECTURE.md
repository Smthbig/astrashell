# AstraShell Architecture Document

## Next-Generation Android Linux Runtime

### Executive Summary

AstraShell is a hybrid Android Linux runtime that combines three execution
modes to deliver near-native Linux performance on stock Android without root:

1. **Native Mode** - LD_PRELOAD + seccomp user notification for CLI tools
2. **Container Mode** - Full Linux namespace isolation for services/daemons
3. **MicroVM Mode** - AVF-backed hardware virtualization for maximum isolation

### Architecture Overview

```
┌──────────────────────────────────────────────────────┐
│                   Android App (Kotlin)                │
│  ┌─────────┐ ┌──────────┐ ┌────────┐ ┌───────────┐  │
│  │ Terminal│ │Container │ │Images  │ │Settings   │  │
│  │ Screen  │ │Screen    │ │Screen  │ │Screen     │  │
│  └────┬────┘ └────┬─────┘ └───┬────┘ └─────┬─────┘  │
│       │           │           │            │         │
│  ┌────┴───────────┴───────────┴────────────┴─────┐  │
│  │            JNI Bridge Layer                     │  │
│  └────────────────────┬───────────────────────────┘  │
├───────────────────────┼──────────────────────────────┤
│  ┌────────────────────┴───────────────────────────┐  │
│  │        Native Runtime (Rust)                    │  │
│  │                                                 │  │
│  │  ┌────────────┐  ┌──────────┐  ┌───────────┐   │  │
│  │  │Engine Mgr  │  │Namespace │  │Syscall    │   │  │
│  │  │(Auto-      │  │Manager   │  │Intercept  │   │  │
│  │  │ detect)    │  │          │  │(seccomp)  │   │  │
│  │  └─────┬──────┘  └────┬─────┘  └─────┬─────┘   │  │
│  │        │              │              │          │  │
│  │  ┌─────┴──────────────┴──────────────┴──────┐   │  │
│  │  │         Execution Pipeline                │   │  │
│  │  └─────┬──────────────┬──────────────┬───────┘   │  │
│  │        │              │              │           │  │
│  │  ┌─────┴────┐  ┌─────┴──────┐  ┌────┴──────┐    │  │
│  │  │Native    │  │Container   │  │AVF MicroVM│    │  │
│  │  │Engine    │  │Engine      │  │Engine     │    │  │
│  │  └──────────┘  └────────────┘  └───────────┘    │  │
│  └─────────────────────────────────────────────────┘  │
├───────────────────────────────────────────────────────┤
│                    Linux Kernel (Android)              │
│  ┌─────────┐ ┌──────────┐ ┌────────┐ ┌────────────┐  │
│  │seccomp  │ │Namespaces│ │Overlay │ │AVF/crosvm  │  │
│  │notify   │ │user/pid/ │ │fs/FUSE │ │(KVM/pKVM)  │  │
│  │         │ │net/uts   │ │        │ │            │  │
│  └─────────┘ └──────────┘ └────────┘ └────────────┘  │
└───────────────────────────────────────────────────────┘
```

### Hybrid Execution Model

```
                    ┌─────────────────────┐
                    │  Workload Analysis   │
                    └──────────┬──────────┘
                               │
                    ┌──────────┴──────────┐
                    │  Auto-detect best   │
                    │  execution mode     │
                    └──────────┬──────────┘
                               │
          ┌────────────────────┼────────────────────┐
          │                    │                    │
   ┌──────┴──────┐    ┌───────┴───────┐    ┌───────┴──────┐
   │  NATIVE     │    │  CONTAINER    │    │   MICROVM    │
   │             │    │              │    │              │
   │ CLI tools   │    │ systemd      │    │ Kernel dev   │
   │ compilers   │    │ Docker       │    │ Full distro  │
   │ scripting   │    │ services     │    │ GUI apps     │
   │ node/python │    │ databases    │    │ GPU workloads│
   │ git/vim     │    │ network apps │    │ untrusted    │
   │             │    │              │    │ code         │
   └─────────────┘    └──────────────┘    └──────────────┘
```

### Syscall Interception Strategy

```
┌──────────────────────────────────────────────────────┐
│                 Guest Process                          │
│  ┌────────────────────────────────────────────────┐  │
│  │  libastrashell-preload.so (LD_PRELOAD)          │  │
│  │  - open() → path translation                    │  │
│  │  - execve() → ABI adaptation                    │  │
│  │  - uname() → hostname spoofing                  │  │
│  │  - fork() → bionic-compatible clone             │  │
│  └─────────────────┬──────────────────────────────┘  │
│                    │ syscall                           │
└────────────────────┼──────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────┐
│           seccomp BPF Filter                          │
│                                                       │
│  mount/umount → SECCOMP_RET_USER_NOTIF → broker daemon│
│  capget/capset → SECCOMP_RET_USER_NOTIF → fake root   │
│  sethostname  → SECCOMP_RET_USER_NOTIF → allow        │
│  all others   → SECCOMP_RET_ALLOW                     │
└────────────────────┬──────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────┐
│           Syscall Broker Daemon (Rust)                │
│                                                       │
│  - Validates mount targets against allowlist          │
│  - Performs privileged operations on behalf of guest  │
│  - Returns fake capabilities for root emulation       │
│  - Monitors via Unix domain socket                    │
└──────────────────────────────────────────────────────┘
```

### Namespace Isolation Model

```
┌──────────────────────────────────────────────────────┐
│                   Android Host                         │
│  UID: ${app_uid}, PID: ${app_pid}                     │
│  SELinux: untrusted_app                               │
│                                                        │
│  ┌────────────────────────────────────────────────┐   │
│  │          User Namespace (CLONE_NEWUSER)         │   │
│  │  UID 0 (root) ↔ UID ${app_uid} (host)          │   │
│  │  GID 0 (root) ↔ GID ${app_gid} (host)          │   │
│  │  Full capabilities inside namespace             │   │
│  │                                                  │   │
│  │  ┌──────────────────────────────────────────┐   │   │
│  │  │     PID Namespace (CLONE_NEWPID)         │   │   │
│  │  │  PID 1 = init/systemd                    │   │   │
│  │  │  Isolated process tree                    │   │   │
│  │  │                                              │   │
│  │  │  ┌──────────────────────────────────────┐   │   │
│  │  │  │   Mount Namespace (CLONE_NEWNS)      │   │   │
│  │  │  │  pivot_root → rootfs                  │   │   │
│  │  │  │  /proc, /sys, /dev (private)          │   │   │
│  │  │  │  overlayfs upper layer (writable)     │   │   │
│  │  │  └──────────────────────────────────────┘   │   │
│  │  │                                              │   │
│  │  │  ┌──────────────────────────────────────┐   │   │
│  │  │  │   Network Namespace (CLONE_NEWNET)   │   │   │
│  │  │  │  slirp4netns → user-mode networking  │   │   │
│  │  │  │  port forwarding to host              │   │   │
│  │  │  └──────────────────────────────────────┘   │   │
│  │  │                                              │   │
│  │  │  ┌──────────────────────────────────────┐   │   │
│  │  │  │   UTS Namespace (CLONE_NEWUTS)       │   │   │
│  │  │  │  hostname: astrashell                 │   │   │
│  │  │  └──────────────────────────────────────┘   │   │
│  │  └──────────────────────────────────────────┘   │   │
│  └────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────┘
```

### Filesystem Layout

```
/data/data/com.astrashell/
├── files/
│   ├── rootfs/              # Base Linux rootfs (read-only)
│   │   ├── bin/
│   │   ├── etc/
│   │   ├── usr/
│   │   ├── lib/
│   │   └── ...
│   ├── upper/               # Overlayfs upper layer (writable)
│   ├── work/                # Overlayfs work dir
│   ├── merged/              # Overlayfs merged view (runtime)
│   ├── images/              # OCI images cache
│   │   ├── arch/
│   │   ├── ubuntu/
│   │   └── alpine/
│   ├── containers/          # Container state
│   ├── snapshots/           # Filesystem snapshots
│   └── cache/               # Package cache
├── databases/                # Runtime databases
│   ├── containers.db
│   └── settings.db
└── lib/
    └── libastrashell.so     # Native runtime library
```

### Performance Comparison

```
Metric              Termux   PRoot    Podroid   Droidspaces   AstraShell
─────────────────────────────────────────────────────────────────────────
Syscall overhead    none     ~1000x    native    native        ~1-2x (notif)
Process isolation  none     weak      VM iso    namespace     hybrid*
Init system        no       broken    systemd   systemd       systemd/OpenRC
Network            native   slow(NAT) QEMU      host/NAT      slirp/host/NAT
GPU                no       no       no        via app       virgl/Wayland
Root emulation     no       ptrace   real(VM)  real(root)    fake root
File access        direct   slow      virtio    overlay       overlay/FUSE
Memory overhead    20MB     50MB     512MB+    100MB         80MB (native)
Boot time          0.5s     1s       20s       2s            0.5s (native)
Docker             no       no       yes       yes           yes
AUR/Arch           no       slow     no        partial       full
Android root req   no       no       no        yes           no

* Hybrid: auto-selects native/container/VM per workload
```

### Module Dependency Graph

```
┌─────────────────────────────────────────────────────────────┐
│                        astrashell                            │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐  │
│  │engine::  │ │engine::  │ │engine::  │ │container::   │  │
│  │native    │ │container │ │vm        │ │oci           │  │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └──────┬───────┘  │
│       │            │            │               │          │
│  ┌────┴────────────┴────────────┴───────────────┴───────┐  │
│  │                   syscall::                           │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐             │  │
│  │  │intercept │ │translate │ │broker    │             │  │
│  │  └──────────┘ └──────────┘ └──────────┘             │  │
│  └──────────────────────┬───────────────────────────────┘  │
│                         │                                  │
│  ┌──────────────────────┴───────────────────────────────┐  │
│  │                   namespace::                         │  │
│  │  ┌──────┐ ┌─────┐ ┌─────┐ ┌──────┐ ┌─────┐ ┌────┐ │  │
│  │  │user  │ │mount│ │pid  │ │net   │ │uts  │ │ipc │ │  │
│  │  └──────┘ └─────┘ └─────┘ └──────┘ └─────┘ └────┘ │  │
│  └──────────────────────┬───────────────────────────────┘  │
│                         │                                  │
│  ┌──────────────────────┴───────────────────────────────┐  │
│  │  fs::     │  net::    │  gui::    │  security::      │  │
│  │ overlay   │  slirp    │  wayland  │  seccomp          │  │
│  │ snapshot  │  forward  │  x11      │  capabilities     │  │
│  │ image     │  dns      │  web      │  sandbox          │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                            │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  ipc::       │  util::        │  platform::         │   │
│  │  bus         │  kernel        │  android/avf        │   │
│  │  protocol    │  path          │                     │   │
│  │  rpc         │  elf           │                     │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Implementation Phases

#### Phase 1 (MVP) - Weeks 1-4
- [x] Minimum Rust runtime engine selection
- [x] Namespace bootstrap (user, mount, pid, uts)
- [x] LD_PRELOAD shim for glibc compatibility
- [x] seccomp user notification interception
- [x] overlay/FUSE filesystem mounting
- [x] Arch Linux rootfs bootstrap
- [x] Basic Android app with terminal
- [x] Config file parsing

#### Phase 2 - Weeks 5-8
- [ ] systemd/OpenRC PID 1 support
- [ ] OCI image pull and unpack
- [ ] Docker/Podman compatibility
- [ ] Package manager (pacman) support
- [ ] AUR helper integration
- [ ] Service management (start/stop/restart)
- [ ] Port forwarding
- [ ] DNS resolution

#### Phase 3 - Weeks 9-12
- [ ] AVF VirtualizationService integration
- [ ] crosvm VM lifecycle management
- [ ] vsock-based communication
- [ ] Microdroid payload support
- [ ] Wayland compositor bridge
- [ ] virglrenderer GPU acceleration
- [ ] Vulkan/OpenGL forwarding
- [ ] Audio forwarding

#### Phase 4 - Weeks 13-16
- [ ] JIT syscall translation
- [ ] eBPF-assisted monitoring
- [ ] Memory deduplication
- [ ] Page cache sharing
- [ ] Dynamic engine switching
- [ ] Seccomp profile optimization
- [ ] Performance benchmarking
- [ ] Battery-aware scheduling

#### Phase 5 - Weeks 17-20
- [ ] VSCode remote integration
- [ ] Web terminal (ttyd-compatible)
- [ ] SSH daemon with key auth
- [ ] Cloud sync for environments
- [ ] Declarative config (devcontainer.json)
- [ ] Package snapshotting
- [ ] IDE integration plugin
- [ ] Play Store release

### Threat Model

```
┌────────────────────────────────────────────────────────────┐
│                 Threat Analysis                             │
├────────────────────────────────────────────────────────────┤
│ Threat                    │ Mitigation                      │
├───────────────────────────┼─────────────────────────────────┤
│ Escape via kernel bug     │ seccomp + namespace isolation   │
│ TOCTOU on seccomp notify  │ PIN_ARGS (kernel 6.7+)          │
│ SELinux denial            │ SELinux-aware operation         │
│ Container breakout        │ drop capabilities, readonly fs  │
│ Privilege escalation      │ no_new_privs, Landlock          │
│ Network attack            │ slirp isolation, port filter    │
│ Resource exhaustion       │ cgroup limits, OOM protection   │
│ Malicious OCI image       │ signature verification          │
│ Filesystem escape         │ pivot_root, private mounts      │
│ Binary planting           │ immutable base layer            │
└────────────────────────────────────────────────────────────┘
```

### Integration Points

```
┌────────────────────────────────────────────────────────────┐
│               Android Integration Points                    │
├────────────────────────────────────────────────────────────┤
│ API/Surface              │ Purpose                          │
├───────────────────────────┼─────────────────────────────────┤
│ VirtualMachineManager    │ AVF VM lifecycle (API 33+)       │
│ VirtualizationService    │ crosvm management (AIDL)         │
│ SurfaceFlinger           │ Android display compositor       │
│ Hardware Composer        │ GPU acceleration                 │
│ AudioFlinger             │ Audio output                     │
│ InputManager             │ Touch/keyboard events            │
│ NetworkStack             │ Tethering, VPN                   │
│ StorageManager           │ Filesystem access                │
│ ContentResolver          │ SAF document access              │
│ Shizuku API              │ Privileged operations            │
│ NotificationManager      │ Runtime status notifications     │
└────────────────────────────────────────────────────────────┘
```
