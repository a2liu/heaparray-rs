//! Utility structs for reference counting.
//!
//! This module exists to make other reference counting structs easier to implement;
//! i.e. the reference counting itself and associated unsafety is handled here
//! so that the other reference counting structs can just call the API. Since
//! all functions are `#[inline]`, this ends up being a zero-cost abstraction.
use core::cell::Cell;
use core::sync::atomic::{AtomicUsize, Ordering};

/// Utility struct that handles reference counting.
///
/// Implementors should maintain the invariant that clones of a `RefCounter`
/// create a clone of the internal data with the reference count set to 1.
pub trait RefCounter<T> {
    /// Returns a new instance of this reference counter.
    fn new(data: T) -> Self;
    /// Decrements the reference counter by one and returns its current value.
    fn decrement(&self) -> usize;
    /// Increments the reference counter by one and returns its current value.
    fn increment(&self) -> usize;
    /// Returns the reference count associated with this struct.
    fn counter(&self) -> usize;
    /// Returns a reference to the data associated with this struct.
    fn get_data(&self) -> &T;
    /// Returns a mutable reference to the data associated with this struct.
    fn get_data_mut(&mut self) -> &mut T;
}

/// Reference counting struct for non-atomic reference counts.
pub struct RcStruct<T> {
    counter: Cell<usize>,
    pub data: T,
}

impl<T> Clone for RcStruct<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self::new(self.data.clone())
    }
}

impl<T> RefCounter<T> for RcStruct<T> {
    fn new(data: T) -> Self {
        Self {
            counter: Cell::new(1),
            data,
        }
    }
    fn decrement(&self) -> usize {
        self.counter.set(self.counter.get() - 1);
        self.counter.get()
    }
    fn increment(&self) -> usize {
        #[cfg(not(feature = "ref-counter-skip-overflow-check"))]
        assert!(
            self.counter.get() < core::usize::MAX,
            "Incrementing the reference count of an `RcStruct`\
             past `core::usize::MAX` is unsafe and results in undefined behavior"
        );
        self.counter.set(self.counter.get() + 1);
        self.counter.get()
    }
    fn counter(&self) -> usize {
        self.counter.get()
    }
    fn get_data(&self) -> &T {
        &self.data
    }
    fn get_data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

/// Reference counting struct for atomic reference counts.
pub struct ArcStruct<T> {
    ref_count: AtomicUsize,
    pub data: T,
}

impl<T> Clone for ArcStruct<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self::new(self.data.clone())
    }
}

impl<T> RefCounter<T> for ArcStruct<T> {
    fn new(data: T) -> Self {
        Self {
            ref_count: AtomicUsize::new(1),
            data,
        }
    }
    fn decrement(&self) -> usize {
        self.ref_count.fetch_sub(1, Ordering::AcqRel) - 1
    }
    fn increment(&self) -> usize {
        #[cfg(not(feature = "ref-counter-skip-overflow-check"))]
        assert!(
            self.counter() < core::usize::MAX,
            "Incrementing the reference count of an `ArcStruct`\
             past `core::usize::MAX` is unsafe and results in undefined behavior"
        );
        self.ref_count.fetch_add(1, Ordering::Relaxed) + 1
    }
    fn counter(&self) -> usize {
        self.ref_count.load(Ordering::Acquire)
    }
    fn get_data(&self) -> &T {
        &self.data
    }
    fn get_data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

unsafe impl<T> Send for ArcStruct<T> where T: Send {}

unsafe impl<T> Sync for ArcStruct<T> where T: Sync {}
