use super::fat::*;
use super::thin::*;
use crate::mem_block::*;

pub struct ThinPtrArrayIterOwned<'a, E, L>(MemBlockIterOwned<'a, E, LenLabel<L>>);

pub struct ThinPtrArrayIterRef<'a, E, L>(MemBlockIterRef<'a, E, LenLabel<L>>);
pub struct ThinPtrArrayIterMut<'a, E, L>(MemBlockIterMut<'a, E, LenLabel<L>>);
