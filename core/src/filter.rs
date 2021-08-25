//! Filter

use crate::geometry::*;
use crate::pbrt::*;
use std::sync::Arc;

/// Filter interface.
pub trait Filter {
    /// Return the filter parameters.
    fn get_data(&self) -> &FilterData;

    /// Returns value of the filter at a given point.
    ///
    /// * `p` - The position of the sample point relative to the center of the
    ///         filter. The point should be within the filter's extent.
    fn evaluate(&self, p: &Point2f) -> Float;
}

/// Atomic reference counted `Filter`.
pub type ArcFilter = Arc<dyn Filter + Send + Sync>;

/// Data for filters centered at origin (0, 0).
pub struct FilterData {
    /// Radius of the filter in x and y directions; beyond this filter is 0.
    pub radius: Vector2f,

    /// Reciprocal of filter radii.
    pub inv_radius: Vector2f,
}

impl FilterData {
    /// Returns a new instance of `FilterData`.
    ///
    /// * `radius` - Radius of the filter in x and y directions; beyond this
    ///              filter is 0.
    pub fn new(radius: Vector2f) -> Self {
        Self {
            radius,
            inv_radius: Vector2f::new(1.0 / radius.x, 1.0 / radius.y),
        }
    }
}
