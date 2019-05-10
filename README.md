---
---
# HeapArray
This crate aims to give people better control of how they want to allocate memory,
by providing a customizable way to allocate blocks of memory, that optionally contains
metadata about the block itself.

## Features
-  Arrays are allocated on the heap
-  Swap owned objects in and out with `array.insert()`

## Examples

Creating an array:
```rust
use heaparray::*;
let len = 10;
let label = ();
let generator = |_label, _idx| 12;
let array = HeapArray::new(label, len, generator);
```

Indexing works as you would expect:
```rust
let d = array[i];
array[i] = 2;
```

