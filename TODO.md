# TODO
- [X] Separate `LabelledArray` into `LabelledArray` and `LabelledArrayMut`,
  and remove `Array` requirement from `LabelledArray` (change it to `CopyMap`)
- [X] Remove implementations of `IndexMut`, `get_label_mut`, etc. from
  `heaparray::naive_rc::generic::RcArray`
- [ ] Slice support, to make iterator implementation reaaaaaally easy.
  - [ ] Use slices for borrow and mutable borrow iteration, and a custom
    struct for owned iteration.
- [ ] Get implementations of standard things like iteration and `Debug` output.
- [X] Get a single implementation of memory block. This allows for
  a more explicit description of the memory layout and a consolidation of
  existing code into a more manageable generalization. Also reduces usage
  of `unsafe` keyword.
- [X] Write pointer types to arrays that are easier to use than raw references
  - [X] `clone_from` for both  
  - [X] Write a modified global allocator to handle allocations during testing  
        **Using existing crate:** https://github.com/a1liu/interloc
  - [X] Write tests
  - [ ] Write test generator on traits, because there are gonna be a bunch of
    implementations of the same traits
    **Status:** *delayed by lack of support*
- [X] Write naive reference counting structs (only strong references) and naive
      atomic reference counting structs
  - [X] `ArrayRef::clone()` to be more idiomatic with `ArrayRef::clone<A>(arr: A) -> A where A: ArrayRef + Clone`
  - [X] Check for nulls, add panics, etc.
  - [ ] Add an interface between ref counted version and normal versions
    **Status:** *delayed, seeming lack of practical utility*
  - [X] Write tests, testing memory usage during clones
- [ ] Write structs that are reference counted. Use naive Rc structs as weak-pointers
  - [ ] Write tests
- [ ] Move to `#![no_std]`
- [ ] Allow the user to customize allocator
  - [ ] Write tests
- [ ] Completely unchecked arrays whose size is never known and whose state needs
  to be manually handled. Purpose is two-fold: makes it possible to turn `ThinPtrArray`
  into a special case of a more general struct; also makes it *really* easy to write
  the constant-sized arrays (as they're another special case)
- [ ] Constant-sized arrays whose size is known at compile time.  
      **Blocked by:** *const generics*
  - [ ] Write tests

## Brainstorming
-  `HeapArray`, `FatPtrArray`, and `ThinPtrArray` are references that are tied to
   their cooresponding memory blocks. They should be treated like the actual memory
   that they represent.
-  `RcArray`, `RcFatPtrArray`, and `RcThinPtrArray` are references that are not tied
   to their cooresponding memory blocks, but are tied to a thread of execution.
   They should be treated like references to objects in a GC language like Java,
   with the caveat that they cannot be sent to other threads safely.
-  `ArcArray`, `ArcFatPtrArray`, and `ArcThinPtrArray` are references that are not
   tied to their cooresponding memory blocks, or their current thread of execution.
   The should be treated like references to objects in a GC language like Java.
