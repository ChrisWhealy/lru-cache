use std::num::{NonZero, NonZeroUsize};

pub const CACHE_SIZES: [NonZero<usize>; 3] = [
    NonZeroUsize::new(1000).unwrap(),
    NonZeroUsize::new(5000).unwrap(),
    NonZeroUsize::new(10000).unwrap(),
];
