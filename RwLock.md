Implicit reader Explicit writer Lock
Struct is:
- writer Mutex<()> for claiming to be a writer. If you want to write you first have to
  grab hold of this mutex.
- permissions RwLock<Option<ThreadId>> for validating that you're a writer. If you
  want to write you then write your ThreadId into this RwLock, and then release it.
- allocation RwLock<()> for preventing wayward allocations. If you want to write
  you finally grab a hold of this lock and then you can write exclusively to memory,
  and all the other threads will be asleep.

When you let go of the lock, you first throw away the allocation lock, so that the
sleeping "reader" allocations can start to happen. Then you write the value
`None` to the RwLock, and then allow for reads, so that there aren't any false
reads. Finally, you discard the writer Mutex, so that other writers can do the
same thing you just did.

The manager checks to see whether the permissions RwLock is correct, and if so
it will try to take the allocation RwLock<()> as a writer, without using RAII.
The only lock that should be handled with RAII is the permissions lock, whose
role is *actually* to synchronize a data structure.

