# HeapArray
This crate aims to give people better control of how they want to allocate memory,
by providing a customizable way to allocate blocks of memory, that optionally contains
metadata about the block itself.

## Features
- Arrays are allocated on the heap, with optional extra space allocated for metadata
- An array implementation that uses an atomic pointer (`heaparray::thin_array_ptr::ThinPtrArray`)
- 1-word and 2-word references to arrays
- Atomically reference-counted memory blocks of arbitrary size without using a `Vec`
- Swap owned objects in and out with `array.insert()`
- Arbitrarily sized objects using label and an array of bytes (`u8`)

## Examples

Creating an array:
```rust
use heaparray::*;
let len = 10;
let array = HeapArray::new(len, |idx| idx + 3);
assert!(array[1] == 4);
```

Indexing works as you would expect:
```rust
array[3] = 2;
assert!(array[3] == 2);
```

Notably, you can take ownership of objects back from the container:

```rust
let mut array = HeapArray::new(10, |_| Vec::<u8>::new());
let replacement_object = Vec::new();
let owned_object = array.insert(0, replacement_object);
```

but you need to give the array a replacement object to fill its slot with.

Additionally, you can customize what information should be stored alongside the elements in
the array using the HeapArray::new_labelled function:

```rust
struct MyLabel {
    pub even: usize,
    pub odd: usize,
}

let mut array = HeapArray::new_labelled(
    MyLabel { even: 0, odd: 0 },
    100,
    |label, index| {
        if index % 2 == 0 {
            label.even += 1;
            index
        } else {
            label.odd += 1;
            index
        }
    });
```

## Use of `unsafe` Keyword
This library relies heavily on the use of the `unsafe` keyword to do both reference
counting and atomic operations; there are 14 instances total, not including tests.

## Future Plans
Iteration, allocator customization, reference counting (`RcArray`),
and atomic reference counting (`ArcArray`).  Constant-sized array of arbitrary size,
i.e. `CArray`, with sizes managed by the type system (waiting on const generics for this one).
See `TODO.md` in the repository for a full list of planned features.

