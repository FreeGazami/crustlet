.PHONY: build

build:
	cargo build

clean:
	cargo clean
	rm -f ./*.img ./*.log ./*.bin