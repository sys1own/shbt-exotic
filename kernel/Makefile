CC ?= gcc
CFLAGS ?= -std=c11 -O2 -ffreestanding -fno-builtin -nostdlib \
	-fpic -Wall -Wextra -Iinclude
# AVX-512 when the toolchain/host supports it; the kernel carries a scalar
# fallback for targets without it.
AVX512 := $(shell printf 'int main(void){return 0;}' | $(CC) -mavx512f -x c - -o /dev/null 2>/dev/null && echo yes)

ifeq ($(AVX512),yes)
CFLAGS += -mavx512f
endif

BUILD := ../build
OBJ := $(BUILD)/shbt_exotic_kernel.o

all: $(BUILD)/shbt_exotic_reference.so

$(BUILD):
	mkdir -p $(BUILD)

$(OBJ): src/shbt_exotic_kernel.c include/shbt_exotic_hardware.h | $(BUILD)
	$(CC) $(CFLAGS) -c $< -o $@

$(BUILD)/shbt_exotic_reference.so: $(OBJ) linker.ld
	$(CC) -shared -nostdlib -T linker.ld -o $@ $(OBJ)

clean:
	rm -rf $(BUILD)

.PHONY: all clean
