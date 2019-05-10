---
---
# TODO
- [X] Write pointer types to arrays that are easier to use than raw references
   - [X] Write tests
- [ ] Write structs that are explicity reference counted, with some exposed unsafe
      methods for altering reference count directly.
   - [ ] Write tests
- [ ] Write methods to correctly create Arc versions
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
