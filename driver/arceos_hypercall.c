#include "arceos_hypercall.h"

uint32_t arceos_hypercall_dummy(uint32_t arg0) {
    return arceos_hypercall2(HYPERCALL_ID_DUMMY, arg0, 0);
}
