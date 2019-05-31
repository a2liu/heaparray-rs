//! Utility structs for reference counting.
//!
//! This module exists to make other reference counting structs easier to implement;
//! i.e. the reference counting itself and associated unsafety is handled here
//! so that the other reference counting structs can just call the API. Since
//! all functions are `#[inline]`, this ends up being a zero-cost abstraction.
use core::marker::PhantomData;
use core::sync::atomic::{AtomicUsize, Ordering};

/// Utility struct that handles reference counting.
pub trait RefCounter<T> {
    /// Returns a new instance of this reference counter
    fn new(data: T) -> Self;
    /// Decrements the reference counter by one and returns its current value
    fn decrement(&self) -> usize;
    /// Increments the reference counter by one and returns its current value
    fn increment(&self) -> usize;
    fn counter(&self) -> usize;
    fn get_data(&self) -> &T;
    fn get_data_mut(&mut self) -> &mut T;
}

/// Reference counting struct for non-atomic reference counts.
pub struct RcStruct<T> {
    phantom: PhantomData<*mut u8>,
    counter: usize,
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
            phantom: PhantomData,
            counter: 1,
            data,
        }
    }
    fn decrement(&self) -> usize {
        unsafe {
            *(&self.counter as *const usize as *mut usize) -= 1;
        }
        self.counter
    }
    fn increment(&self) -> usize {
        unsafe {
            *(&self.counter as *const usize as *mut usize) += 1;
        }
        self.counter
    }
    fn counter(&self) -> usize {
        self.counter
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
    counter: AtomicUsize,
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
            counter: AtomicUsize::new(1),
            data,
        }
    }
    fn decrement(&self) -> usize {
        self.counter.fetch_sub(1, Ordering::AcqRel) - 1
    }
    fn increment(&self) -> usize {
        self.counter.fetch_add(1, Ordering::Relaxed) + 1
    }
    fn counter(&self) -> usize {
        self.counter.load(Ordering::Acquire)
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
