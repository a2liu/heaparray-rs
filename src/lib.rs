/*!
This crate aims to give people better control of how they allocate memory,
by providing a customizable way to allocate blocks of memory, that optionally
contains metadata about the block itself. This makes it much easier to implement
Dynamically-Sized Types (DSTs), and also reduces the number of pointer
indirections necessary to share data between threads.

## Features
- Safe API to dynamically-sized types
- Generic implementations of common tasks so you can customize the
  implementation of a type without having to write additional boilerplate
- Atomically reference-counted memory blocks of arbitrary size without
  using a `Vec`; this means you can access reference-counted memory with
  only a single pointer indirection.

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
use heaparray::*;
let mut array = HeapArray::new(10, |_| 0);
array[3] = 2;
assert!(array[3] == 2);
```

Additionally, you can customize what information should be stored alongside
the elements in the array using the `HeapArray::with_label` function:

```rust
# use heaparray::*;
struct MyLabel {
    pub even: usize,
    pub odd: usize,
}

let array = HeapArray::with_label(
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

## Dynamically Sized Types
The [Rust documentation on exotically sized types][rust-docs-dsts],
at the end of the section on dynamically-sized types states that:

[rust-docs-dsts]: https://doc.rust-lang.org/nomicon/exotic-sizes.html

> Currently the only properly supported way to create a custom DST is by
> making your type generic and performing an unsizing coercion...
> (Yes, custom DSTs are a largely half-baked feature for now.)

This crate aims to provide *some* of that functionality; the code that
the docs give is the following:

```rust
struct MySuperSliceable<T: ?Sized> {
    info: u32,
    data: T
}

fn main() {
    let sized: MySuperSliceable<[u8; 8]> = MySuperSliceable {
        info: 17,
        data: [0; 8],
    };

    let dynamic: &MySuperSliceable<[u8]> = &sized;

    // prints: "17 [0, 0, 0, 0, 0, 0, 0, 0]"
    println!("{} {:?}", dynamic.info, &dynamic.data);
}
```

using this crate, the `MySuperSliceable<[u8]>` type would be
implemented like this:

```rust
use heaparray::*;

type MySuperSliceable = HeapArray<u8, u32>;

fn main() {
    let info = 17;
    let len = 8;
    let dynamic = MySuperSliceable::with_label(info, len, |_,_| 0);
    println!("{:?}", dynamic);
}
```
*/

extern crate atomic_types;
extern crate const_utils;
extern crate containers_rs as containers;

mod api;
pub mod base;
pub mod impls;
pub mod naive_rc;
mod traits;

mod api_prelude {
    pub use crate::traits::*;
    pub use containers::{Container, CopyMap};
}

mod api_prelude_rc {
    pub use crate::api_prelude::*;
    pub use crate::traits::rc::*;
}

mod prelude {
    pub use crate::api_prelude::*;
    pub(crate) use core::fmt;
    pub(crate) use core::mem;
    pub(crate) use core::ops::{Index, IndexMut};
}

pub use api::*;
