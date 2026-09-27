#![no_std]

extern crate alloc;

pub fn collect_even(values: &[u32]) -> alloc::vec::Vec<u32> {
    values.iter().copied().filter(|value| value % 2 == 0).collect()
}
