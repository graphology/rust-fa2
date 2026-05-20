use std::ops::{AddAssign, DivAssign, SubAssign};

use num_traits::Float as NumFloat;

pub trait Float: NumFloat + SubAssign + AddAssign + DivAssign {}
impl<T: NumFloat + SubAssign + AddAssign + DivAssign> Float for T {}
