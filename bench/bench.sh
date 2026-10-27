#!/usr/bin/env bash
# Boot time benchmark: ferriboot vs systemd-boot vs Limine vs GRUB.
# QEMU + KVM + OVMF; the same GPT disk, kernel and initramfs for every loader.
# Result: seconds from QEMU start to the kernel's first timestamped log line.
#
#   ./bench/bench.sh                                   all loaders, 15 runs each
#   RUNS=30 ./bench/bench.sh                           more runs
#   LOADERS="ferriboot grub" ./bench/bench.sh          only some loaders
#   SHOW=1 LOADERS=ferriboot RUNS=1 ./bench/bench.sh   with a QEMU window
set -euo pipefail

RUNS=${RUNS:-15}
LOADERS=${LOADERS:-"ferriboot systemd-boot limine grub"}
SHOW=${SHOW:-0}
HERE=$(cd "$(dirname "$0")" && pwd)
FERRIBOOT_EFI=${FERRIBOOT_EFI:-$HERE/../target/x86_64-unknown-uefi/release/bootloader.efi}
KERNEL=$HERE/vmlinuz-linux
INITRD=$HERE/initramfs-linux.img
LOGS=$HERE/logs
OVMF_CODE=/usr/share/edk2/x64/OVMF_CODE.4m.fd
OVMF_VARS=/usr/share/edk2/x64/OVMF_VARS.4m.fd
MNT=/mnt/ferriboot-bench
# earlyprintk + ignore_loglevel: the kernel writes to the serial port as early
# as possible and nothing is filtered out.
# rdinit/root/panic: the kernel panics right after unpacking the initramfs,
# and QEMU exits because of -no-reboot, so every run takes a few seconds.
CMDLINE="earlyprintk=serial,ttyS0,115200 console=ttyS0 ignore_loglevel rdinit=/nonexistent root=/dev/null panic=-1"
# The finish line: the first kernel log line with a [ seconds.micro ] timestamp.
MARKER='\[ *[0-9]+\.[0-9]+\]'

for f in "$FERRIBOOT_EFI" "$KERNEL" "$INITRD" "$OVMF_CODE" "$OVMF_VARS" \
         /usr/lib/systemd/boot/efi/systemd-bootx64.efi /usr/share/limine/BOOTX64.EFI; do
    [[ -r $f ]] || { echo "missing: $f" >&2; exit 1; }
done
for c in qemu-system-x86_64 ts sfdisk mkfs.fat losetup grub-mkstandalone; do
    command -v "$c" > /dev/null || { echo "missing command: $c" >&2; exit 1; }
done

WORK=$(mktemp -d)
LOOP=""
cleanup() {
    sudo umount "$MNT" 2> /dev/null || true
    if [[ -n $LOOP ]]; then
        sudo losetup -d "$LOOP" 2> /dev/null || true
    fi
    rm -rf "$WORK"
}
trap cleanup EXIT

sudo -v
mkdir -p "$LOGS"
mkdir -p "$WORK"/{ferriboot,systemd-boot,limine,grub}/EFI/BOOT

# --- ferriboot ---
cp "$FERRIBOOT_EFI" "$WORK/ferriboot/EFI/BOOT/BOOTX64.EFI"
mkdir -p "$WORK/ferriboot/EFI/ferriboot"
cat > "$WORK/ferriboot/EFI/ferriboot/ferriboot.conf" <<EOF
timeout: 0
default: 1

/Arch Linux
protocol: linux
kernel: \\vmlinuz-linux
initrd: \\initramfs-linux.img
cmdline: $CMDLINE
EOF

# --- systemd-boot ---
cp /usr/lib/systemd/boot/efi/systemd-bootx64.efi "$WORK/systemd-boot/EFI/BOOT/BOOTX64.EFI"
mkdir -p "$WORK/systemd-boot/loader/entries"
printf 'timeout 0\ndefault arch.conf\n' > "$WORK/systemd-boot/loader/loader.conf"
cat > "$WORK/systemd-boot/loader/entries/arch.conf" <<EOF
title Arch Linux
linux /vmlinuz-linux
initrd /initramfs-linux.img
options $CMDLINE
EOF

# --- Limine ---
cp /usr/share/limine/BOOTX64.EFI "$WORK/limine/EFI/BOOT/BOOTX64.EFI"
cat > "$WORK/limine/limine.conf" <<EOF
timeout: 0

/Arch Linux
    protocol: linux
    path: boot():/vmlinuz-linux
    module_path: boot():/initramfs-linux.img
    cmdline: $CMDLINE
EOF
cp "$WORK/limine/limine.conf" "$WORK/limine/EFI/BOOT/limine.conf"

# --- GRUB (standalone image with the config embedded) ---
cat > "$WORK/grub.cfg" <<EOF
insmod part_gpt
insmod fat
set timeout=0
set default=0
search --no-floppy --file --set=root /vmlinuz-linux
menuentry "Arch Linux" {
    linux /vmlinuz-linux $CMDLINE
    initrd /initramfs-linux.img
}
EOF
grub-mkstandalone -O x86_64-efi --modules="part_gpt fat" \
    -o "$WORK/grub/EFI/BOOT/BOOTX64.EFI" "boot/grub/grub.cfg=$WORK/grub.cfg"

# make_disk NAME: GPT disk with one 500 MiB FAT32 ESP holding $WORK/NAME + kernel + initrd
make_disk() {
    local name=$1 img=$WORK/$1.img tree=$WORK/$1
    cp "$KERNEL" "$INITRD" "$tree/"
    truncate -s 600M "$img"
    printf 'label: gpt\nstart=2048, size=500MiB, type=uefi\n' | sfdisk -q "$img"
    LOOP=$(sudo losetup -fP --show "$img")
    sudo mkfs.fat -F 32 "${LOOP}p1" > /dev/null
    sudo mkdir -p "$MNT"
    sudo mount "${LOOP}p1" "$MNT"
    sudo cp -r "$tree/." "$MNT/"
    sync
    sudo umount "$MNT"
    sudo losetup -d "$LOOP"
    LOOP=""
}

# measure NAME: boots NAME.img once, prints the time of the first kernel line (or nothing)
measure() {
    local img=$WORK/$1.img log=$LOGS/$1.log
    local display=(-display none)
    if [[ $SHOW == 1 ]]; then
        display=()
    fi
    cp "$OVMF_VARS" "$WORK/vars.fd"
    timeout 60 qemu-system-x86_64 -enable-kvm -cpu host -m 2G -smp 2 \
        -no-reboot "${display[@]}" -monitor none \
        -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
        -drive if=pflash,format=raw,file="$WORK/vars.fd" \
        -drive file="$img",format=raw,if=virtio \
        -serial stdio < /dev/null 2>&1 | ts -s '%.s' > "$log" || true
    grep -m1 -E "$MARKER" "$log" | cut -d' ' -f1 || true
}

printf '%-14s %8s %8s %8s  (seconds, %d runs)\n' loader median min max "$RUNS"
for name in $LOADERS; do
    make_disk "$name"
    if [[ $SHOW != 1 ]]; then
        measure "$name" > /dev/null   # warm-up, discarded
    fi
    first=$(measure "$name")
    if [[ -z $first ]]; then
        printf '%-14s FAILED, see %s\n' "$name" "$LOGS/$name.log"
        continue
    fi
    times=("$first")
    for ((i = 1; i < RUNS; i++)); do
        t=$(measure "$name")
        if [[ -n $t ]]; then
            times+=("$t")
        fi
    done
    printf '%s\n' "${times[@]}" | sort -n | awk -v n="$name" '
        { a[NR] = $1 }
        END { printf "%-14s %8.3f %8.3f %8.3f\n", n, a[int((NR + 1) / 2)], a[1], a[NR] }'
done
