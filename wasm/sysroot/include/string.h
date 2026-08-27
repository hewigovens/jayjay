#ifndef JAYJAY_WASM_STRING_H_
#define JAYJAY_WASM_STRING_H_

#include_next <string.h>

#ifndef NULL
#define NULL ((void *)0)
#endif

static inline void *memchr(const void *src, int c, size_t count) {
  const unsigned char *bytes = src;
  for (size_t i = 0; i < count; i++) {
    if (bytes[i] == (unsigned char)c) return (void *)&bytes[i];
  }
  return NULL;
}

static inline char *strchr(const char *text, int c) {
  while (*text != (char)c) {
    if (*text == '\0') return NULL;
    text++;
  }
  return (char *)text;
}

static inline int strcmp(const char *left, const char *right) {
  while (*left && *left == *right) {
    left++;
    right++;
  }
  return *(const unsigned char *)left - *(const unsigned char *)right;
}

char *strncpy(char *restrict dest, const char *restrict src, size_t count);

#endif
