use std::ops::{AddAssign, DivAssign, SubAssign};

use num_traits::{Float as NumFloat, FloatConst};

pub trait Float: NumFloat + FloatConst + SubAssign + AddAssign + DivAssign + Send + Sync {}
impl<T: NumFloat + FloatConst + SubAssign + AddAssign + DivAssign + Send + Sync> Float for T {}
