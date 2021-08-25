//! Light Types
/// Stores combination of flags for the light types.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct LightType {
    t: u8,
}

/// Types of lights.
pub const DELTA_POSITION_LIGHT: u8 = 1;
pub const DELTA_DIRECTION_LIGHT: u8 = 2;
pub const AREA_LIGHT: u8 = 4;
pub const INFINITE_LIGHT: u8 = 8;

impl LightType {
    /// Tests a single light type flag and returns whether it is set or not.
    ///
    /// * `flag` - Light type flag.
    pub fn matches(&self, flag: u8) -> bool {
        self.t & flag > 0
    }

    /// Returns true if the light flags has the DELTA_POSITION_LIGHT or
    /// DELTA_DIRECTION_LIGHT flag set.
    ///
    /// * `flags` - Light flags to check.
    pub fn is_delta_light(&self) -> bool {
        self.t & DELTA_POSITION_LIGHT > 0 || self.t & DELTA_DIRECTION_LIGHT > 0
    }
}

impl PartialEq for LightType {
    /// Returns true if the light types match.
    ///
    /// * `other` - The light type to compare.
    fn eq(&self, other: &Self) -> bool {
        self.t & other.t == self.t
    }
}

impl From<u8> for LightType {
    /// Convert a `u8` value to `LightType`.
    ///
    /// * `t` - A `u8` value containing combination of `*_LIGHT` flags combined
    ///         bitwise OR operator.
    fn from(t: u8) -> Self {
        assert!(
            t <= INFINITE_LIGHT,
            "Invalid light flags {}=({:#08b})",
            t,
            t
        );
        Self { t }
    }
}
