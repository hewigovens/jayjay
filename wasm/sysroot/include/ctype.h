#ifndef JAYJAY_WASM_CTYPE_H_
#define JAYJAY_WASM_CTYPE_H_

#include_next <ctype.h>

static inline int isalpha(int c) {
  return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z');
}

static inline int isdigit(int c) {
  return c >= '0' && c <= '9';
}

static inline int isalnum(int c) {
  return isalpha(c) || isdigit(c);
}

static inline int isspace(int c) {
  return c == ' ' || c == '\t' || c == '\n' || c == '\v' || c == '\f' || c == '\r';
}

static inline int ispunct(int c) {
  return isprint(c) && !isalnum(c) && !isspace(c);
}

static inline int tolower(int c) {
  return c >= 'A' && c <= 'Z' ? c + ('a' - 'A') : c;
}

static inline int toupper(int c) {
  return c >= 'a' && c <= 'z' ? c - ('a' - 'A') : c;
}

#endif
