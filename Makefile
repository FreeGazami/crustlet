.PHONY: build

ifneq (,$(wildcard ./.env))
include .env
export
endif

build:
	cargo build --target x86_64-unknown-uefi
ifdef PACKAGE_NAME
	qemu-img create -f raw $(PACKAGE_NAME).img 64M
	mkfs.fat -F 32 $(PACKAGE_NAME).img
	mkdir efi_mount/
	sudo mount -o loop $(PACKAGE_NAME).img efi_mount/
	sudo mkdir -p efi_mount/EFI/BOOT
	sudo cp target/x86_64-unknown-uefi/debug/$(PACKAGE_NAME).efi efi_mount/EFI/BOOT/BOOTX64.EFI
	sudo umount efi_mount
else
	@echo "did not find PACKAGE_NAME defined. Skipping efi disk image generation"
endif

clean:
	cargo clean

print-env:
	@echo "PACKAGE_NAME: $(PACKAGE_NAME)"
	@echo "PACKAGE_VERSION: $(PACKAGE_VERSION)"
	@echo "BIN_NAME: $(BIN_NAME)"