//! Gaussian Filter

use core::filter::*;
use core::geometry::*;
use core::paramset::*;
use core::pbrt::*;

/// Implements the Gaussian filter which applies a bump that is centered at the
/// pixel and radially symmetric around it.
pub struct GaussianFilter {
    /// Filter data.
    pub data: FilterData,

    /// Falloff rate.
    pub alpha: Float,

    /// Stores e^(-alpha * radius.x^2).
    pub exp_x: Float,

    /// Stores e^(-alpha * radius.y^2).
    pub exp_y: Float,
}

impl GaussianFilter {
    /// Returns a new instance of `GaussianFilter`.
    ///
    /// * `radius` - Radius of the filter in x and y directions; beyond this
    ///              filter is 0.
    /// * `alpha`  - Falloff rate.
    pub fn new(radius: Vector2f, alpha: Float) -> Self {
        Self {
            data: FilterData::new(radius),
            alpha,
            exp_x: (-alpha * radius.x * radius.x).exp(),
            exp_y: (-alpha * radius.y * radius.y).exp(),
        }
    }

    /// Calculates the Gaussian filter function for a given distance.
    ///
    /// * `d`    - Distance in x or y direction.
    /// * `expv` - Corresponding exponent `exp_x` or `exp_y`.
    fn gaussian(&self, d: Float, expv: Float) -> Float {
        max(0.0, (-self.alpha * d * d).exp() - expv)
    }
}

impl Filter for GaussianFilter {
    /// Return the filter parameters.
    fn get_data(&self) -> &FilterData {
        &self.data
    }

    /// Returns value of the filter at a given point.
    ///
    /// * `p` - The position of the sample point relative to the center of the
    ///         filter. The point should be within the filter's extent.
    fn evaluate(&self, p: &Point2f) -> Float {
        self.gaussian(p.x, self.exp_x) * self.gaussian(p.y, self.exp_y)
    }
}

impl From<&ParamSet> for GaussianFilter {
    /// Create a `GaussianFilter` from `ParamSet`.
    ///
    /// * `params` - Parameter set.
    fn from(params: &ParamSet) -> Self {
        let xw = params.find_one_float("xwidth", 2.0);
        let yw = params.find_one_float("ywidth", 2.0);
        let alpha = params.find_one_float("alpha", 2.0);
        Self::new(Vector2f::new(xw, yw), alpha)
    }
}
