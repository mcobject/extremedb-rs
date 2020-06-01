// util.rs
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

use std::ops::{BitAnd, BitAndAssign, BitOrAssign, Not};

pub(crate) struct BitMask<T>(T);

impl<T> BitMask<T>
where
    T: BitAnd<Output = T>
        + BitAndAssign
        + BitOrAssign
        + Default
        + Not<Output = T>
        + Copy
        + PartialEq,
{
    pub(crate) fn new() -> Self {
        Self(T::default())
    }

    pub(crate) fn from(val: T) -> Self {
        Self(val)
    }

    pub(crate) fn bit_mask(&self) -> T {
        self.0
    }

    pub(crate) fn set_bit_value(&mut self, f: T, v: bool) {
        match v {
            true => self.set(f),
            false => self.reset(f),
        }
    }

    pub(crate) fn get_bit_value(&self, f: T) -> bool {
        (self.0 & f) != T::default()
    }

    fn set(&mut self, f: T) {
        self.0 |= f;
    }

    fn reset(&mut self, f: T) {
        self.0 &= !f;
    }
}

pub(crate) type BitMask32 = BitMask<u32>;
