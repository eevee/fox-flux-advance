NAME := fox-flux-advance

all: debug

.PHONY: debug
debug:
	mkdir -p target
	arm-none-eabi-as crt0.s -o target/crt0.o
	cargo xbuild --target thumbv4-none-agb.json
	arm-none-eabi-objcopy -O binary target/thumbv4-none-agb/debug/$(NAME) target/$(NAME).gba
	gbafix target/$(NAME).gba
