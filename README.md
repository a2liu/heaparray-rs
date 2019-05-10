---
---
# HeapArray
This crate aims to give people better control of how they want to allocate memory,
by providing a customizable way to allocate blocks of memory, that optionally contains
metadata about the block itself.

It's suggested that you import the contents of this crate with `use heaparray::*;`, as a lot of the functionality of the structs in this crate are
implemented through traits.

## Features
-  Arrays are allocated on the heap
-  Swap owned objects in and out with `array.insert()`

## Examples

Creating an array:

```rust
use heaparray::*;
let len = 10;
let label = ();
let generator = |_label, idx| idx + 3;
let array = HeapArray::new(label, len, generator);
```

Indexing works as you would expect:

```rust
let d = array[i];
array[i] = 2;
```

Notably, you can take ownership of objects back from the container:

```rust
let owned_object = array.insert(0, replacement_object);
```

but you need to give the array a replacement object to fill its slot with.
