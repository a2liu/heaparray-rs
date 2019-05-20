//! Contains definition of `AtomicPtrArray`, an array reference whose pointer is
//! 1 word and atomically loaded/stored.
//!
//! Generic operations need to be done through atomic loads and stores of the
//! internal pointer value;
