#ifndef JAYJAY_WASM_STDLIB_H_
#define JAYJAY_WASM_STDLIB_H_

#include_next <stdlib.h>

// Swift's scanner exits on allocation failure; a browser module has no process to terminate.
static inline __attribute__((noreturn)) void exit(int status) {
  (void)status;
  abort();
}

#endif
