/*
 * shbt_exotic_hardware.h — SHBT-MMIO-EXOTIC register map.
 *
 * 128-byte dual-cacheline aligned register block anchored at physical base
 * address 0x70000000. Cache line 0 carries control/status and the power
 * ledger; cache line 1 carries the extended exotic vector registers.
 */
#ifndef SHBT_EXOTIC_HARDWARE_H
#define SHBT_EXOTIC_HARDWARE_H

#include <stdint.h>

#define SHBT_EXOTIC_MMIO_BASE 0x70000000UL
#define SHBT_EXOTIC_MMIO_SIZE 128u

/* Register offsets */
#define REG_SYS_CONTROL        0x00 /* Enable | Quench | Superpos | Transloc */
#define REG_SYS_STATUS         0x04 /* Quench latched | ECC corr | 2PN | Lock */
#define REG_POWER_DEBT_KW      0x08 /* Landauer debt, fixed kW (906.00)       */
#define REG_LANR_OUTPUT_KW     0x10 /* Net LANR generation, kW (999.054)      */
#define REG_SEED_MASS_LO       0x18 /* Ghost-seed mass, low dword             */
#define REG_SEED_MASS_HI       0x20 /* Ghost-seed mass, high dword            */
#define REG_DS2_INTERVAL_LO    0x28 /* Signed 2PN interval, low dword         */
#define REG_DS2_INTERVAL_HI    0x2C /* Signed 2PN interval, high dword        */
#define REG_QUENCH_TIME_NS     0x30 /* Hardware latch timer (<= 2.18 ns)      */
#define REG_ANOMALY_FLAGS      0x34 /* Spacelike | underpower | rigidity      */
#define REG_WARP_LAPSE_METRIC  0x38 /* |det(g)+1| lapse metric (Q64.64 hi)    */
#define REG_HEEGAARD_RELABEL   0x40 /* Active Sp(2g,Z) relabel index          */
#define REG_STASIS_DILUTION    0x48 /* Stasis clock dilution factor           */
#define REG_EXOTIC_VEC0        0x50 /* Extended exotic vector 0               */
#define REG_EXOTIC_VEC1        0x58 /* Extended exotic vector 1               */
#define REG_EXOTIC_VEC2        0x60 /* Extended exotic vector 2               */
#define REG_EXOTIC_VEC3        0x68 /* Extended exotic vector 3               */
#define REG_EXOTIC_VEC4        0x70 /* Extended exotic vector 4               */
#define REG_CRC32_CASTAGNOLI   0x7C /* Frame CRC-32C checksum                 */

/* REG_SYS_CONTROL bits */
#define CTRL_ENABLE            (1u << 0)
#define CTRL_QUENCH_TRIGGER    (1u << 1)
#define CTRL_SUPERPOSITION     (1u << 2)
#define CTRL_TRANSLOC_ENGAGE   (1u << 3)

/* REG_SYS_STATUS bits */
#define STAT_QUENCH_LATCHED    (1u << 0)
#define STAT_ECC_CORRECTED     (1u << 1)
#define STAT_2PN_AUTHORIZED    (1u << 2)
#define STAT_TRANSLOC_LOCK     (1u << 3)

/* REG_ANOMALY_FLAGS bits */
#define ANOM_SPACELIKE         (1u << 0)
#define ANOM_UNDERPOWER        (1u << 1)
#define ANOM_RIGIDITY_FAULT    (1u << 2)

/*
 * shbt_exotic_mmio_t — packed 128-byte register file. The struct is aligned
 * to a 64-byte boundary so both 64-byte cache lines are independently
 * coherent for zero-copy C-ABI streaming.
 */
typedef struct __attribute__((aligned(64), packed)) {
    /* Cache line 0 (0x00-0x3F) */
    volatile uint32_t sys_control;        /* 0x00 */
    volatile uint32_t sys_status;         /* 0x04 */
    volatile uint32_t power_debt_kw;      /* 0x08 */
    volatile uint32_t _rsv0;              /* 0x0C */
    volatile uint32_t lanr_output_kw;     /* 0x10 */
    volatile uint32_t _rsv1;              /* 0x14 */
    volatile uint32_t seed_mass_lo;       /* 0x18 */
    volatile uint32_t _rsv2;              /* 0x1C */
    volatile uint32_t seed_mass_hi;       /* 0x20 */
    volatile uint32_t _rsv3;              /* 0x24 */
    volatile uint32_t ds2_interval_lo;    /* 0x28 */
    volatile uint32_t ds2_interval_hi;    /* 0x2C */
    volatile uint32_t quench_time_ns;     /* 0x30 */
    volatile uint32_t anomaly_flags;      /* 0x34 */
    volatile uint32_t warp_lapse_metric;  /* 0x38 */
    volatile uint32_t _rsv4[1];           /* 0x3C */
    /* Cache line 1 (0x40-0x7F) */
    volatile uint32_t heegaard_relabel;   /* 0x40 */
    volatile uint32_t _rsv5;              /* 0x44 */
    volatile uint32_t stasis_dilution;    /* 0x48 */
    volatile uint32_t _rsv6;              /* 0x4C */
    volatile uint64_t exotic_vec[5];      /* 0x50-0x77 */
    volatile uint32_t _rsv7;              /* 0x78 */
    volatile uint32_t crc32_castagnoli;   /* 0x7C */
} shbt_exotic_mmio_t;

_Static_assert(sizeof(shbt_exotic_mmio_t) == 128, "MMIO block must be 128 B");

#define SHBT_EXOTIC_MMIO ((shbt_exotic_mmio_t *)SHBT_EXOTIC_MMIO_BASE)

#endif /* SHBT_EXOTIC_HARDWARE_H */
