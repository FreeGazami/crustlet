.PHONY: build create-img clean run-qemu

ifneq (,$(wildcard ./.env))
include .env
export
endif

IMG_NAME=$(PACKAGE_NAME)-$(TRIPLE).img

all: run-qemu

build:
	cargo build

# Creates a FAT32 image for UEFI boot
create-img: build
	qemu-img create -f raw $(IMG_NAME) 64M
	mkfs.fat -F 32 $(IMG_NAME)
	mkdir -p efi_mount
	sudo mount -o loop $(IMG_NAME) efi_mount
	sudo mkdir -p efi_mount/EFI/BOOT
	sudo mkdir -p efi_mount/crustlet/
	sudo cp target/x86_64-unknown-uefi/debug/$(PACKAGE_NAME).efi efi_mount/EFI/BOOT/BOOTX64.EFI
	sudo cp runtime_configs/rEnv.txt efi_mount/crustlet/rEnv.txt
	sudo cp kernel/gazami efi_mount/gazami
	sudo umount efi_mount
	rm -rf efi_mount

run-qemu: create-img
	qemu-system-x86_64 -bios /usr/share/ovmf/x64/OVMF.4m.fd -drive file=$(IMG_NAME),format=raw -m 4G

clean:
	cargo clean
	rm -f ./*.img