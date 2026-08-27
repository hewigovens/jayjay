#ifndef JAYJAY_WASM_WCTYPE_H_
#define JAYJAY_WASM_WCTYPE_H_

#include <stdbool.h>
#include_next <wctype.h>

typedef __WCHAR_TYPE__ wchar_t;

static inline int iswlower(wint_t c) {
  return c >= L'a' && c <= L'z';
}

static inline int iswupper(wint_t c) {
  return c >= L'A' && c <= L'Z';
}

static inline int iswpunct(wint_t c) {
  return c >= 0x20 && c <= 0x7e && !iswalnum(c) && !iswspace(c);
}

static inline int iswxdigit(wint_t c) {
  return (c >= L'0' && c <= L'9') || (c >= L'a' && c <= L'f') || (c >= L'A' && c <= L'F');
}

static inline wint_t towlower(wint_t c) {
  return iswupper(c) ? c + (L'a' - L'A') : c;
}

static inline wint_t towupper(wint_t c) {
  return iswlower(c) ? c - (L'a' - L'A') : c;
}

#endif
