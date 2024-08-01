use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Angle {
    pub radians: f32,
}
impl Angle {
    pub fn new(radians: f32) -> Self {
        Self { radians }
    }

    pub fn from_degrees(degrees: f32) -> Self {
        Self::new(degrees.to_radians())
    }

    pub fn to_degrees(self) -> f32 {
        self.radians.to_degrees()
    }

    pub fn to_radians(self) -> f32 {
        self.radians
    }

    pub fn min(self, other: Self) -> Self {
        Self {
            radians: self.radians.min(other.radians),
        }
    }

    pub fn max(self, other: Self) -> Self {
        Self {
            radians: self.radians.max(other.radians),
        }
    }
}

impl Add for Angle {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            radians: self.radians + rhs.radians,
        }
    }
}

impl Sub for Angle {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            radians: self.radians - rhs.radians,
        }
    }
}
