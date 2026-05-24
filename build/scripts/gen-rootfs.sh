#!/bin/bash
# Generate Arch Linux ARM64 rootfs for AstraShell
set -euo pipefail

ROOTFS_DIR="${1:-./rootfs}"
MIRROR="${2:-http://mirror.archlinuxarm.org}"

echo "=== Generating Arch Linux ARM64 rootfs ==="
echo "Output: $ROOTFS_DIR"

# Create rootfs directory
mkdir -p "$ROOTFS_DIR"

# Bootstrap Arch Linux ARM
if [ ! -f /usr/bin/arch-chroot ]; then
    echo "arch-install-scripts required. Install with: apt install arch-install-scripts"
    exit 1
fi

# Bootstrap
echo "Bootstrapping base system..."
cat > /tmp/pacstrap.conf << 'EOF'
[options]
Architecture = aarch64
SigLevel = Optional TrustAll

[core]
Server = http://mirror.archlinuxarm.org/aarch64/core

[extra]
Server = http://mirror.archlinuxarm.org/aarch64/extra

[community]
Server = http://mirror.archlinuxarm.org/aarch64/community

[alarm]
Server = http://mirror.archlinuxarm.org/aarch64/alarm
EOF

# Install base packages
pacstrap -C /tmp/pacstrap.conf -cGM "$ROOTFS_DIR" \
    base base-devel linux-aarch64 \
    pacman openssh sudo vim tmux \
    python nodejs go rust \
    gcc clang llvm \
    git curl wget \
    ca-certificates \
    networkmanager \
    dbus

# Configure
echo "Configuring rootfs..."
arch-chroot "$ROOTFS_DIR" /bin/bash << 'CHROOT'
    # Setup locale
    echo "en_US.UTF-8 UTF-8" >> /etc/locale.gen
    locale-gen
    echo "LANG=en_US.UTF-8" > /etc/locale.conf

    # Setup timezone
    ln -sf /usr/share/zoneinfo/UTC /etc/localtime

    # Setup hostname
    echo "astrashell" > /etc/hostname

    # Setup hosts
    cat > /etc/hosts << 'HOSTS'
127.0.0.1   localhost
::1         localhost
127.0.1.1   astrashell.localdomain astrashell
HOSTS

    # Create user
    useradd -m -G wheel -s /bin/bash astra
    echo "astra:astra" | chpasswd
    echo "root:root" | chpasswd

    # Setup sudo
    echo "%wheel ALL=(ALL) ALL" >> /etc/sudoers

    # Enable services
    systemctl enable sshd
    systemctl enable NetworkManager
    systemctl enable dbus

    # Setup pacman for AUR
    sed -i 's/#Color/Color/' /etc/pacman.conf
    echo '[multilib]' >> /etc/pacman.conf
    echo 'Include = /etc/pacman.d/mirrorlist' >> /etc/pacman.conf

    # Clean package cache
    pacman -Scc --noconfirm
CHROOT

# Create AstraShell integration
mkdir -p "$ROOTFS_DIR/opt/astrashell"
cat > "$ROOTFS_DIR/opt/astrashell/astrashell_env.sh" << 'ENV'
export PATH="/opt/astrashell/bin:$PATH"
export ASTRASHELL_ENGINE="native"
ENV

# Package rootfs
echo "Packaging rootfs..."
cd "$(dirname "$ROOTFS_DIR")"
tar -czf "astrashell-arch-rootfs.tar.gz" "$(basename "$ROOTFS_DIR")"

echo ""
echo "=== Rootfs generated ==="
echo "Rootfs: $ROOTFS_DIR"
echo "Archive: $(dirname "$ROOTFS_DIR")/astrashell-arch-rootfs.tar.gz"
echo "Size: $(du -sh "$ROOTFS_DIR" | cut -f1)"
