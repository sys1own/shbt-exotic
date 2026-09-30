/*
 * reference_test.c — hosted reference test for the shbt-exotic C11
 * microkernel (shbt-qc tests/reference_test.c lineage).
 *
 * Compiled against kernel/include/shbt_exotic_hardware.h and
 * kernel/src/shbt_exotic_kernel.c (the kernel TU is #included so the
 * static SECDED/Givens/quench internals are reachable; _start is renamed
 * away via -D_start=kernel_boot_start).
 *
 * Validates:
 *   - 128-byte shbt_exotic_mmio_t alignment contract
 *   - SECDED Hamming(72,64): single-bit flip corrected w/ syndrome set,
 *     double-bit flip flagged uncorrectable (DUE)
 *   - Givens vector remapping preserves the vector norm (AVX-512 or
 *     scalar fallback path)
 *   - PCSS crowbar trigger latches sub-2.5 ns
 */
#include <assert.h>
#include <stdio.h>
#include <stdint.h>
#include <math.h>
#include <sys/mman.h>

/* Arena symbols the kernel expects from linker.ld — provided locally. */
uint8_t __stinespring_start[2112];
uint8_t __active_end[1];
uint8_t __stinespring_end[1];

#include "../kernel/src/shbt_exotic_kernel.c"

/* Reference decode of a Hamming(72,64) codeword, mirroring the kernel's
 * parity_pos pattern. Returns syndrome via *syn and overall parity via
 * *par; the corrected data word is returned. */
static uint64_t ref_decode(__uint128_t code, uint64_t *syn, int *par)
{
    static const uint8_t parity_pos[7] = {0, 1, 3, 7, 15, 31, 63};
    uint64_t syndrome = 0, overall = 0, data = 0;
    unsigned d = 0;

    for (unsigned i = 0; i < 7; ++i) {
        uint64_t p = parity_pos[i] + 1, parity = 0;
        for (uint64_t pos = 1; pos <= 71; ++pos)
            if (pos & p)
                parity ^= (code >> (pos - 1)) & 1ull;
        syndrome |= parity << i;
    }
    for (unsigned i = 0; i < 72; ++i)
        overall ^= (code >> i) & 1ull;
    if (syndrome && overall)
        code ^= (__uint128_t)1 << (syndrome - 1);
    for (uint64_t pos = 1; pos <= 71; ++pos)
        if ((pos & (pos - 1)) != 0)
            data |= (uint64_t)((code >> (pos - 1)) & (__uint128_t)1) << d++;
    *syn = syndrome;
    *par = (int)overall;
    return data;
}

int main(void)
{
    /* 1. 128-byte register block. */
    assert(sizeof(shbt_exotic_mmio_t) == 128);

    /* Map the fixed MMIO page so the kernel can touch 0x70000000. */
    void *page = mmap((void *)SHBT_EXOTIC_MMIO_BASE, 4096,
                      PROT_READ | PROT_WRITE,
                      MAP_PRIVATE | MAP_ANONYMOUS | MAP_FIXED, -1, 0);
    assert(page == (void *)SHBT_EXOTIC_MMIO_BASE);

    /* 2. SECDED Hamming(72,64). */
    {
        uint64_t data = 0xDEADBEEF12345678ull;
        __uint128_t code = secded_encode(data);
        uint64_t syn; int par;

        /* Clean codeword decodes to the same data. */
        assert(ref_decode(code, &syn, &par) == data && syn == 0 && par == 0);

        /* Single-bit flip: corrected, syndrome set, parity odd. */
        code ^= (__uint128_t)1 << 20;
        assert(ref_decode(code, &syn, &par) == data && syn != 0 && par == 1);

        /* Double-bit flip: uncorrectable (DUE) — syndrome nonzero while
         * overall parity is even; the kernel flags this as a panic trap. */
        code = secded_encode(data) ^ ((__uint128_t)3 << 20);
        ref_decode(code, &syn, &par);
        assert(syn != 0 && par == 0);
    }

    /* 3. Givens vector remapping preserves the norm. */
    {
        double x[8] = {1, 2, 3, 4, 5, 6, 7, 8};
        double y[8] = {8, 7, 6, 5, 4, 3, 2, 1};
        double n0 = 0.0, n1 = 0.0;
        for (int i = 0; i < 8; ++i) n0 += x[i] * x[i] + y[i] * y[i];
        givens_remap(x, y, 8, 0.6, 0.8);
        for (int i = 0; i < 8; ++i) n1 += x[i] * x[i] + y[i] * y[i];
        assert(fabs(n1 - n0) < 1e-9);
    }

    /* 4. PCSS crowbar trigger: latches under the 2.50 ns hard limit and
     * sets STAT_QUENCH_LATCHED in the register file. */
    shbt_exotic_kernel_init();
    {
        double latency = quench_trigger();
        assert(latency < PCSS_HARD_LIMIT_NS);
        assert(mmio->sys_status & STAT_QUENCH_LATCHED);
        assert(latency <= 2.18 + 1e-9);   /* 8 x 0.27 ns stages */
    }

    /* 5. One service pass: ECC scrub over the 2112 B arena, Givens remap
     * of the braid descriptors, CRC-32C trailer update. */
    shbt_exotic_service();
    assert(mmio->crc32_castagnoli != 0);

    puts("reference_test: all checks passed");
    return 0;
}
