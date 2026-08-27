#ifndef JAYJAY_WASM_ASSERT_H_
#define JAYJAY_WASM_ASSERT_H_

#ifdef NDEBUG
#define assert(expression) ((void)0)
#else
static inline __attribute__((noreturn)) void __assert_fail(
    const char *assertion,
    const char *file,
    unsigned line,
    const char *function) {
  (void)assertion;
  (void)file;
  (void)line;
  (void)function;
  __builtin_trap();
}
#define assert(expression) \
  ((expression) ? (void)0 : __assert_fail(#expression, __FILE__, __LINE__, __func__))
#endif

#ifndef static_assert
#define static_assert _Static_assert
#endif

#endif
