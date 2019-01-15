NAME := fox-flux-advance
ARCH := thumbv4-none-agb

all: debug

.PHONY: target
target:
	@mkdir -p target/assets

.PHONY: debug
debug: target/$(ARCH)/debug/$(NAME).gba

target/$(ARCH)/debug/$(NAME).gba: target target/crt0.o target/assets/lexy.bin target/assets/tiles.bin
	cargo xbuild --target build-stuff/gba/$(ARCH).json
	arm-none-eabi-objcopy -O binary target/$(ARCH)/debug/$(NAME) target/$(ARCH)/debug/$(NAME).gba
	gbafix target/$(ARCH)/debug/$(NAME).gba

.PHONY: release
release: target/$(ARCH)/release/$(NAME).gba
	
target/$(ARCH)/release/$(NAME).gba: target target/crt0.o target/assets/lexy.bin target/assets/tiles.bin
	cargo xbuild --release --target build-stuff/gba/$(ARCH).json
	arm-none-eabi-objcopy -O binary target/$(ARCH)/release/$(NAME) target/$(ARCH)/release/$(NAME).gba
	gbafix target/$(ARCH)/release/$(NAME).gba

# Use order only dependancies to force the target folder to exist without rebuilding them all the time
# http://www.gnu.org/software/make/manual/make.html#Prerequisite-Types
target/crt0.o: | target
	arm-none-eabi-as build-stuff/gba/crt0.s -o target/crt0.o

target/assets/lexy.bin: build-stuff/png-to-tiles.py assets/lexy.png | target
	python build-stuff/png-to-tiles.py assets/lexy.png 32 64 -o target/assets/lexy.bin

target/assets/tiles.bin: build-stuff/png-to-tiles.py assets/tiles.png | target
	python build-stuff/png-to-tiles.py assets/tiles.png 8 8 -o target/assets/tiles.bin
