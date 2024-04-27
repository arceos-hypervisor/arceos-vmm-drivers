#ifndef _ARCEOS_HYPERCALL_H
#define _ARCEOS_HYPERCALL_H 1

#include <linux/types.h>

/**
 * Invoke a hypercall.
 *
 * We follow the convention from RVM to save hypercall id in eax and first
 * two args in edi and esi, the result will be in eax.
 * 
 * However, we allow more (up to 5) arguments to be passed to the hypercall.
 * The last 3 arguments will be passed in edx, ecx, ebx respectively.
 */
static inline uint32_t arceos_hypercall(uint32_t id, uint32_t arg0, uint32_t arg1, uint32_t arg2, uint32_t arg3, uint32_t arg4) {
    uint32_t result;

    asm volatile(
        "vmcall"
        : "=a"(result)
        : "a"(id), "D"(arg0), "S"(arg1), "d"(arg2), "c"(arg3), "b"(arg4)
        : "memory"
    );

    return result;
}

/**
 * Invoke a hypercall with 2 arguments. For compatibility with RVM.
 */
static inline uint32_t arceos_hypercall2(uint32_t id, uint32_t arg0, uint32_t arg1) {
    return arceos_hypercall(id, arg0, arg1, 0, 0, 0);
}

#define HYPERCALL_ID_DUMMY 0

uint32_t arceos_hypercall_dummy(uint32_t arg0);

#endif
