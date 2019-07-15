# Changelog

## 0.5.2
- Added `RcArray::clone` as an intrinsic, which clones the data instead of the reference
- Added implementation of `Eq` and `PartialEq` for arrays
- Added `RcArray::ref_eq` method to check if two references point to the same data

## 0.5.1
- Made `RcArray` available for use.

## 0.5.0
- The `MemBlock` struct handles allocating a block of memory to hold an arbitrary
  dynamically sized type
- The `BaseArray` struct handles the logic of using a pointer to an array-like
  block of memory, but without any overhead for length checking
- The `SafeArray` struct handles length checking, providing a safe API to the
  `BaseArray` struct
- The `HeapArray` struct is a concrete variant of the `SafeArray` struct, and is
  the default implementation of a pointer to an array

