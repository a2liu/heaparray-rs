//! Utility structs for reference counting.
//!
//! This module exists to make other reference counting structs easier to implement;
//! i.e. the reference counting itself and associated unsafety is handled here
//! so that the other reference counting structs can just call the API. Since
//! all functions are `#[inline]`, this ends up being a zero-cost abstraction.
use core::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Reference counting struct for non-atomic reference counts.
pub struct RcStruct<T> {
    phantom: PhantomData<*mut u8>,
    counter: usize,
    pub data: T,
}

impl<T> RcStruct<T> {
    /// Returns a new instance of this reference counter
    #[inline]
    pub fn new(data: T) -> Self {
        Self {
            phantom: PhantomData,
            counter: 1,
            data,
        }
    }
    /// Decrements the reference counter by one and returns its current value
    #[inline]
    pub fn decrement(&self) -> usize {
        unsafe {
            *(&self.counter as *const usize as *mut usize) -= 1;
        }
        self.counter
    }
    /// Increments the reference counter by one and returns its current value
    #[inline]
    pub fn increment(&self) -> usize {
        unsafe {
            *(&self.counter as *const usize as *mut usize) += 1;
        }
        self.counter
    }
}

/// Reference counting struct for atomic reference counts.
pub struct ArcStruct<T> {
    counter: AtomicUsize,
    pub data: T,
}

impl<T> ArcStruct<T> {
    /// Returns a new instance of this atomic reference counter
    #[inline]
    pub fn new(data: T) -> Self {
        Self {
            counter: AtomicUsize::new(1),
            data,
        }
    }
    /// Atomically decrements the reference counter by one and returns its current value.
    #[inline]
    pub fn decrement(&self) -> usize {
        self.counter.fetch_sub(1, Ordering::Relaxed) - 1
    }
    /// Atomically increments the reference counter by one and returns its current value
    #[inline]
    pub fn increment(&self) -> usize {
        self.counter.fetch_add(1, Ordering::Relaxed) + 1
    }
}
