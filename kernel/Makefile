CC ?= gcc
CFLAGS ?= -std=c11 -O2 -ffreestanding -fno-builtin -nostdlib \
	-fpic -Wall -Wextra -Iinclude
# AVX-512 when the toolchain/host supports it; the kernel carries a scalar
# fallback for targets without it.
AVX512 := $(shell printf 'int main(void){return 0;}' | $(CC) -mavx512f -x c - -o /dev/null 2>/dev/null && echo yes)
AVX512_MFLAG := $(if $(filter yes,$(AVX512)),-mavx512f,)

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

$(BUILD)/reference_test: ../tests/reference_test.c src/shbt_exotic_kernel.c include/shbt_exotic_hardware.h | $(BUILD)
	$(CC) -std=c11 -O2 -Wall -Wextra -Iinclude -I../tests \
		-D_start=kernel_boot_start -D_GNU_SOURCE $(AVX512_MFLAG) \
		../tests/reference_test.c -lm -o $@

test-c: $(BUILD)/reference_test
	./$(BUILD)/reference_test

clean:
	rm -rf $(BUILD)

.PHONY: all clean test-c
