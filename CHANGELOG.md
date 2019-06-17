# Changelog

## 0.5.0
- The `MemBlock` struct handles allocating a block of memory to hold an arbitrary
  dynamically sized type
- The `BaseArray` struct handles the logic of using a pointer to an array-like
  block of memory, but without any overhead for length checking
- The `SafeArray` struct handles length checking, providing a safe API to the
  `BaseArray` struct
- The `HeapArray` struct is a concrete variant of the `SafeArray` struct, and is
  the default implementation of a pointer to an array

