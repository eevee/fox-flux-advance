NAME := fox-flux-advance
ARCH := thumbv4-none-agb

all: debug

.PHONY: target
target:
	@mkdir -p target/assets

.PHONY: debug
debug: target/$(ARCH)/debug/$(NAME).gba

target/$(ARCH)/debug/$(NAME).gba: target target/crt0.o target/assets/lexy.bin target/assets/terrain.bin
	cargo xbuild --target build-stuff/gba/$(ARCH).json
	arm-none-eabi-objcopy -O binary target/$(ARCH)/debug/$(NAME) target/$(ARCH)/debug/$(NAME).gba
	gbafix target/$(ARCH)/debug/$(NAME).gba

.PHONY: release
release: target/$(ARCH)/release/$(NAME).gba
	
target/$(ARCH)/release/$(NAME).gba: target target/crt0.o target/assets/lexy.bin target/assets/terrain.bin
	cargo xbuild --release --target build-stuff/gba/$(ARCH).json
	arm-none-eabi-objcopy -O binary target/$(ARCH)/release/$(NAME) target/$(ARCH)/release/$(NAME).gba
	gbafix target/$(ARCH)/release/$(NAME).gba

# FIXME all of these need the target dir to exist, but i don't want to depend on it because it's phony so these will always run, but i rely on that above to force cargo to always run...
target/crt0.o:
	arm-none-eabi-as build-stuff/gba/crt0.s -o target/crt0.o

target/assets/lexy.bin: build-stuff/png-to-tiles.py assets/lexy.png
	python build-stuff/png-to-tiles.py assets/lexy.png 32 64 -o target/assets/lexy.bin

target/assets/terrain.bin: build-stuff/png-to-tiles.py assets/terrain.png
	python build-stuff/png-to-tiles.py assets/terrain.png 32 32 -o target/assets/terrain.bin
