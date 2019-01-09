NAME := fox-flux-advance
ARCH := thumbv4-none-agb

all: debug

target:
	@mkdir -p target/assets

.PHONY: debug
debug: target/$(ARCH)/debug/$(NAME).gba

target/$(ARCH)/debug/$(NAME).gba: target/crt0.o target/assets/lexy.bin target/assets/terrain.bin
	cargo xbuild --target build-stuff/gba/$(ARCH).json
	arm-none-eabi-objcopy -O binary target/$(ARCH)/debug/$(NAME) target/$(ARCH)/debug/$(NAME).gba
	gbafix target/$(ARCH)/debug/$(NAME).gba

.PHONY: release
release: target/$(ARCH)/release/$(NAME).gba
	
target/$(ARCH)/release/$(NAME).gba: target/crt0.o target/assets/lexy.bin target/assets/terrain.bin
	cargo xbuild --release --target build-stuff/gba/$(ARCH).json
	arm-none-eabi-objcopy -O binary target/$(ARCH)/release/$(NAME) target/$(ARCH)/release/$(NAME).gba
	gbafix target/$(ARCH)/release/$(NAME).gba

target/crt0.o: target
	arm-none-eabi-as build-stuff/gba/crt0.s -o target/crt0.o

target/assets/lexy.bin: target build-stuff/png-to-tiles.py assets/lexy.png
	python build-stuff/png-to-tiles.py assets/lexy.png 32 64 -o target/assets/lexy.bin

target/assets/terrain.bin: target build-stuff/png-to-tiles.py assets/terrain.png
	python build-stuff/png-to-tiles.py assets/terrain.png 32 32 -o target/assets/terrain.bin
