---
---
# TODO
- [X] Write pointer types to arrays that are easier to use than raw references
   - [ ] `clone_from` for both
   - [ ] Write a modified global allocator to handle allocations during testing
   - [X] Write tests
- [ ] Write naive reference counting structs (only strong references) and naive
      atomic reference counting structs
   - [ ] `Ref::clone()` and `AtomicRef::clone()` to be more idiomatic
   - [ ] Write tests
- [ ] Write structs that are explicity reference counted, with some exposed unsafe
      methods for altering reference count directly. Use naive Rc structs as weak-pointers
   - [ ] Write tests
- [ ] Write methods to correctly create Arc versions, same strategy as before
   - [ ] Write tests
- [ ] Allow the user to customize allocator
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
