# TODO
- [X] Write pointer types to arrays that are easier to use than raw references
  - [ ] `clone_from` for both  
        **Status:** *delayed*
  - [X] Write a modified global allocator to handle allocations during testing  
        **Using existing crate:** https://github.com/neoeinstein/stats_alloc/
  - [X] Write tests
- [ ] Write naive reference counting structs (only strong references) and naive
      atomic reference counting structs
  - [X] `ArrayRef::clone()` to be more idiomatic with `ArrayRef::clone<A>(arr: A) -> A where A: ArrayRef + Clone`
  - [ ] Check for nulls, add panics, etc.
  - [ ] Add an interface between ref counted version and normal versions
  - [ ] Write tests, testing memory usage during clones
- [ ] Write structs that are reference counted. Use naive Rc structs as weak-pointers
  - [ ] Write tests
- [ ] Write methods to correctly create Arc versions, same strategy as before
  - [ ] Write tests
- [ ] Allow the user to customize allocator
  - [ ] Write tests
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
