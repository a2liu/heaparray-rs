# Code Organization
All library source code is held in the `src` directory. Test source code is held in
the `tests` directory. `mod.rs` files have been excluded, as have any files currently
not used by the library.

```
src
├── api.rs <------------------- Imports objects for `heaparray::*`.
├── base <--------------------- The bare minimum necessary to implement an array on the heap.
│   ├── alloc_utils.rs <--------- Utilities for allocating memory.
│   ├── base.rs <---------------- Defines `BaseArray`.
│   ├── mem_block.rs <----------- Defines `MemBlock`.
│   └── traits.rs <-------------- Defines traits that act as interfaces to `BaseArray`.
├── impls <-------------------- Implements safe array types.
│   ├── generic.rs <------------- Defines `SafeArray`.
│   └── p_types.rs <------------- Defines pointer types that work with `SafeArray`.
├── lib.rs <------------------- The starting point of the library.
├── naive_rc <----------------- Implements safe reference counting types.
│   ├── generic.rs <------------- Defines `RcArray`.
│   ├── ref_counters.rs <-------- Defines reference counting structs.
│   └── types.rs <--------------- Defines more user-friendly versions of `RcArray`.
└── traits <------------------- Contains the traits this library uses.
    ├── array_ref.rs <----------- Defines `ArrayRef` trait.
    ├── labelled_array.rs <------ Defines `LabelledArray` & `LabelledArrayMut` traits.
    ├── make_array.rs <---------- Defines `MakeArray` trait.
    └── slice_array.rs <--------- Defines `SliceArray` & `SliceArrayMut` traits.


tests
├── memory_model <------------- Testing the allocation and deallocation methods.
│   ├── base_array.rs <---------- Tests that `BaseArray` works as expected.
│   ├── mem_block.rs <----------- Tests that `MemBlock` works as expected.
│   └── test_utils.rs <---------- Utilities to check for correct deallocation.
└── memory_model_test.rs <----- Imports memory_model module.
```
