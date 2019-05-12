static IGNORE_LOCK: AtomicBool = AtomicBool::new(false);
static EXCLUSIVE: AtomicBool = AtomicBool::new(false);

#[allow(dead_code)]
struct OldTestMonitor {
    pub reader_count: AtomicUsize,
}

#[allow(dead_code)]
impl OldTestMonitor {
    pub fn new() -> Self {
        Self {
            reader_count: AtomicUsize::new(0),
        }
    }

    // Credit goes to https://stackoverflow.com/questions/27791532/how-do-i-create-a-global-mutable-singleton
    fn lock() -> &'static RwLock<Option<ThreadId>> {
        static mut LOCK: *const RwLock<Option<ThreadId>> = 0 as *const RwLock<Option<ThreadId>>;
        static INIT_LOCK: AtomicBool = AtomicBool::new(false);
        static INIT_READY: AtomicBool = AtomicBool::new(false);

        if INIT_READY.load(Ordering::Acquire) {
            unsafe { &*LOCK }
        } else if INIT_LOCK.compare_and_swap(false, true, Ordering::Relaxed) {
            while !INIT_READY.load(Ordering::Acquire) {
                // std::thread::sleep(std::time::Duration::new(0, 1));
            }
            unsafe { &*LOCK }
        } else {
            IGNORE_LOCK.store(true, Ordering::SeqCst);
            let lock = RwLock::<Option<ThreadId>>::new(None);
            unsafe { LOCK = Box::into_raw(Box::new(lock)) as *const RwLock<Option<ThreadId>> };
            IGNORE_LOCK.store(false, Ordering::Release);
            INIT_READY.store(true, Ordering::Release);
            unsafe { &*LOCK }
        }
    }

    fn lock_pointer(&self) -> &'static RwLock<Option<ThreadId>> {
        Self::lock()
    }

    pub fn gain_exclusive() {
        while !EXCLUSIVE.compare_and_swap(false, true, Ordering::Acquire) {}
        let id = std::thread::current().id();
        *Self::lock().write().unwrap() = Some(id);
    }

    pub fn release_exclusive() {
        let option_id = { *Self::lock().read().unwrap() };
        let id = match option_id {
            Some(id) => id,
            None => panic!("This shouldn't happen!"),
        };
        assert!(id == std::thread::current().id());
        *Self::lock().write().unwrap() = None;
        EXCLUSIVE.store(false, Ordering::Release);
    }
}

impl AllocMonitor for OldTestMonitor where {
    /// Read-write lock because I smartn't. The invariant that should hold is
    /// that any non-exclusive use that goes through must first see a `None`,
    /// and then must increment the reader counter. When finished, using the
    /// allocator, they then decrement the counter.
    fn monitor(&self, _info: AllocInfo, _layout: Layout, act: AllocAction) {
        use std::thread;
        if IGNORE_LOCK.load(Ordering::Acquire) {
            return;
        } else if act.relation() == AllocRel::Before {
            let mut exclusive_writer = false;
            if EXCLUSIVE.load(Ordering::SeqCst) {
                loop {
                    let option_id = { *self.lock_pointer().read().unwrap() };
                    if let Some(id) = option_id {
                        if id == thread::current().id() {
                            exclusive_writer = true;
                            break;
                        } else {
                            thread::sleep(std::time::Duration::new(1, 0));
                        }
                    } else {
                        break;
                    }
                }
            }
            if exclusive_writer {
                while self.reader_count.load(Ordering::SeqCst) != 0 {
                    // thread::sleep(std::time::Duration::new(0, 1));
                }
            } else {
                self.reader_count.fetch_add(1, Ordering::SeqCst);
            }
        } else {
            let option_id = { *self.lock_pointer().read().unwrap() };
            if let Some(id) = option_id {
                if id != thread::current().id() {
                    self.reader_count.fetch_sub(1, Ordering::SeqCst);
                }
            } else {
                self.reader_count.fetch_sub(1, Ordering::SeqCst);
            }
        }
    }
}
