//! RGB Spectrum.

use super::*;
use crate::pbrt::*;
use std::convert::TryInto;
use std::fmt;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

/// Number of spectral samples to use for `RGBSpectrum`.
pub const RGB_SAMPLES: usize = 3;

/// RGBSpectrum represents an spectral power distribution (SPD) with
/// a weighted sum of red, green and blue components.
#[derive(Copy, Clone, PartialEq)]
pub struct RGBSpectrum {
    /// The sampled spectral values.
    c: [Float; 3],
}

impl RGBSpectrum {
    /// Create a new `RGBSpectrum` with a constant value across all
    /// wavelengths.
    ///
    /// * `v` - Constant value.
    pub fn new(v: Float) -> Self {
        let ret = Self {
            c: [v; RGB_SAMPLES],
        };
        assert!(!ret.has_nans());
        ret
    }

    /// Spectrum with all values set to 0.
    pub const ZERO: Self = Self { c: [0.0, 0.0, 0.0] };

    /// Spectrum with all values set to 1.
    pub const ONE: Self = Self { c: [1.0, 1.0, 1.0] };
}

impl Default for RGBSpectrum {
    /// Return a black `RGBSpectrum`.
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<Float> for RGBSpectrum {
    /// Create a new `RGBSpectrum` with a constant value across all
    /// wavelengths.
    ///
    /// * `v` - Constant value.
    fn from(v: Float) -> Self {
        Self::new(v)
    }
}

impl From<Vec<Float>> for RGBSpectrum {
    /// Create a new `RGBSpectrum` from sampled spectral values.
    ///
    /// * `c` - Sample values.
    fn from(c: Vec<Float>) -> Self {
        let ret = Self {
            c: c.try_into().unwrap_or_else(|v: Vec<Float>| {
                panic!(
                    "Expected a Vec of length {} but it was {}",
                    RGB_SAMPLES,
                    v.len()
                )
            }),
        };
        assert!(!ret.has_nans());
        ret
    }
}

impl From<[Float; RGB_SAMPLES]> for RGBSpectrum {
    /// Create a new `RGBSpectrum` from sampled spectral values.
    ///
    /// * `c` - Sample values.
    fn from(c: [Float; RGB_SAMPLES]) -> Self {
        let ret = Self { c };
        assert!(!ret.has_nans());
        ret
    }
}

impl From<&Vec<Sample>> for RGBSpectrum {
    /// Create a new `RGBSpectrum` from spectral samples.
    ///
    /// * `samples` - Samples.
    fn from(samples: &Vec<Sample>) -> Self {
        // Sort samples if unordered.
        let mut sorted_samples = samples.clone();
        if !are_spectrum_samples_sorted(samples) {
            sort_spectrum_samples(&mut sorted_samples);
        };

        let mut xyz = [0.0; 3];
        for i in 0..CIE_SAMPLES {
            let val = interpolate_spectrum_samples(samples, (CIE_LAMBDA_START + i) as Float);
            xyz[0] += val * CIE_X[i];
            xyz[1] += val * CIE_Y[i];
            xyz[2] += val * CIE_Z[i];
        }

        let scale =
            (CIE_LAMBDA_END - CIE_LAMBDA_START) as Float / (CIE_Y_INTEGRAL * CIE_SAMPLES as Float);
        xyz[0] *= scale;
        xyz[1] *= scale;
        xyz[2] *= scale;

        Self::from_xyz(&xyz, None)
    }
}

impl CoefficientSpectrum for RGBSpectrum {
    /// Returns the stored samples.
    fn samples(&self) -> &[Float] {
        &self.c
    }

    /// Returns stored samples as mutable.
    fn samples_mut(&mut self) -> &mut [Float] {
        &mut self.c
    }

    /// Converts XYZ values to a full SPD.
    ///
    /// * `xyz`           - XYZ colour value.
    /// * `spectrum_type` - Indicates type of colour value. If `None`,
    ///                     defaults to `SpectrumType::Reflectance`.
    fn from_xyz(xyz: &[Float; 3], _spectrum_type: Option<SpectrumType>) -> Self {
        Self { c: xyz_to_rgb(xyz) }
    }

    /// Convert the SPD to XYZ cooefficients.
    fn to_xyz(&self) -> [Float; 3] {
        rgb_to_xyz(&self.c)
    }

    /// Returns the y-coefficient of XYZ colour.
    fn y(&self) -> Float {
        0.212671 * self.c[0] + 0.715160 * self.c[1] + 0.072169 * self.c[2]
    }

    /// Converts RGB values to a full SPD.
    ///
    /// * `rgb`           - RGB colour value.
    /// * `spectrum_type` - Indicates type of colour value. If `None`,
    ///                     defaults to `SpectrumType::Reflectance`.
    fn from_rgb(rgb: &[Float; 3], _spectrum_type: Option<SpectrumType>) -> Self {
        Self {
            c: [rgb[0], rgb[1], rgb[2]],
        }
    }

    /// Convert the SPD to RGB cooefficients.
    fn to_rgb(&self) -> [Float; 3] {
        [self.c[0], self.c[1], self.c[2]]
    }

    /// Takes the square root of all sample values.
    fn sqrt(&self) -> Self {
        Self {
            c: [self.c[0].sqrt(), self.c[1].sqrt(), self.c[2].sqrt()],
        }
    }

    /// Raises the sample values to a given power.
    ///
    /// * `p` - The power.
    fn pow(&self, p: Float) -> Self {
        Self {
            c: [self.c[0].powf(p), self.c[1].powf(p), self.c[2].powf(p)],
        }
    }

    /// Converts to an `RGBSpectrum`.
    fn to_rgb_spectrum(&self) -> RGBSpectrum {
        *self
    }

    /// Sets sample values `v` to `e^v`.
    ///
    /// * `other` - The other SPD.
    fn exp(&self) -> Self {
        let mut ret = *self;
        for s in ret.samples_mut().iter_mut() {
            *s = s.exp();
        }
        debug_assert!(!ret.has_nans());
        ret
    }
}

impl From<SampledSpectrum> for RGBSpectrum {
    /// Create a new `RGBSpectrum` from `SampledSpectrum`.
    ///
    /// * `s` - The `SampledSpectrum`.
    fn from(s: SampledSpectrum) -> Self {
        RGBSpectrum { c: s.to_rgb() }
    }
}

impl Add for RGBSpectrum {
    type Output = Self;

    /// Adds the corresponding sample values from another `RGBSpectrum`.
    ///
    /// * `other` - The other `RGBSpectrum`.
    fn add(self, other: Self) -> Self::Output {
        let mut ret = self;
        CoefficientSpectrum::add(&mut ret, &other);
        ret
    }
}

impl AddAssign for RGBSpectrum {
    /// Adds the corresponding sample values from another `RGBSpectrum`.
    ///
    /// * `other` - The other `RGBSpectrum`.
    fn add_assign(&mut self, other: Self) {
        CoefficientSpectrum::add(self, &other);
    }
}

impl Sub for RGBSpectrum {
    type Output = Self;

    /// Subtracts the corresponding sample values from another `RGBSpectrum`.
    ///
    /// * `other` - The other `RGBSpectrum`.
    fn sub(self, other: Self) -> Self::Output {
        let mut ret = self;
        CoefficientSpectrum::sub(&mut ret, &other);
        ret
    }
}

impl SubAssign for RGBSpectrum {
    /// Subtracts the corresponding sample values from another `RGBSpectrum`.
    ///
    /// * `other` - The other `RGBSpectrum`.
    fn sub_assign(&mut self, other: Self) {
        CoefficientSpectrum::sub(self, &other);
    }
}

impl Mul for RGBSpectrum {
    type Output = Self;

    /// Multiplies the corresponding sample values from another `RGBSpectrum`.
    ///
    /// * `other` - The other `RGBSpectrum`.
    fn mul(self, other: Self) -> Self::Output {
        let mut ret = self;
        CoefficientSpectrum::mul(&mut ret, &other);
        ret
    }
}

impl Mul<&RGBSpectrum> for RGBSpectrum {
    type Output = Self;

    /// Multiplies the corresponding sample values from another `RGBSpectrum`.
    ///
    /// * `other` - The other `RGBSpectrum`.
    fn mul(self, other: &RGBSpectrum) -> Self::Output {
        let mut ret = self;
        CoefficientSpectrum::mul(&mut ret, other);
        ret
    }
}

impl Mul<Float> for RGBSpectrum {
    type Output = Self;

    /// Scales the sample values with a constant factor.
    ///
    /// * `f` - Scaling factor.
    fn mul(self, f: Float) -> Self::Output {
        let mut ret = self;
        CoefficientSpectrum::scale(&mut ret, f);
        ret
    }
}

impl Mul<RGBSpectrum> for Float {
    type Output = RGBSpectrum;

    /// Scales the sample values of an `RGBSpectrum`.
    ///
    /// * `s` - Sample values.
    fn mul(self, s: RGBSpectrum) -> Self::Output {
        s * self
    }
}

impl Mul<&RGBSpectrum> for Float {
    type Output = RGBSpectrum;

    /// Scales the sample values of an `RGBSpectrum`.
    ///
    /// * `s` - Sample values.
    fn mul(self, s: &RGBSpectrum) -> Self::Output {
        *s * self
    }
}

impl MulAssign for RGBSpectrum {
    /// Multiplies the corresponding sample values from another `RGBSpectrum`.
    ///
    /// * `other` - The other `RGBSpectrum`.
    fn mul_assign(&mut self, other: Self) {
        CoefficientSpectrum::mul(self, &other);
    }
}

impl MulAssign<Float> for RGBSpectrum {
    /// Scales the sample values with a constant factor.
    ///
    /// * `f` - Scaling factor.
    fn mul_assign(&mut self, f: Float) {
        CoefficientSpectrum::scale(self, f);
    }
}

impl Div for RGBSpectrum {
    type Output = Self;

    /// Divides the corresponding sample values from another
    /// `RGBSpectrum`.
    ///
    /// * `other` - The other `RGBSpectrum`.
    fn div(self, other: Self) -> Self::Output {
        let mut ret = self;
        CoefficientSpectrum::div(&mut ret, &other);
        ret
    }
}

impl Div<Float> for RGBSpectrum {
    type Output = Self;

    /// Divides the sample values with given factor.
    ///
    /// * `f` - Scaling value.
    fn div(self, f: Float) -> Self::Output {
        let mut ret = self;
        CoefficientSpectrum::scale(&mut ret, 1.0 / f);
        ret
    }
}

impl DivAssign for RGBSpectrum {
    /// Divides the corresponding sample values from another `RGBSpectrum`.
    ///
    /// * `other` - The other `RGBSpectrum`.
    fn div_assign(&mut self, other: Self) {
        CoefficientSpectrum::div(self, &other);
    }
}

impl DivAssign<Float> for RGBSpectrum {
    /// Divides the sample values with given factor.
    ///
    /// * `f` - Scaling value.
    fn div_assign(&mut self, f: Float) {
        CoefficientSpectrum::scale(self, 1.0 / f);
    }
}

impl Neg for RGBSpectrum {
    type Output = Self;

    /// Scale the values by -1.
    fn neg(self) -> Self::Output {
        self * -1.0
    }
}

impl Index<usize> for RGBSpectrum {
    type Output = Float;

    /// Index the sample value.
    ///
    /// * `i` -  The index.
    fn index(&self, index: usize) -> &Self::Output {
        &self.c[index]
    }
}

impl IndexMut<usize> for RGBSpectrum {
    /// Index the sample to get a mutable sample value.
    ///
    /// * `i` - The index.
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.c[i]
    }
}

impl Clamp<Float> for RGBSpectrum {
    /// Clamps the sample values.
    ///
    /// * `low`  - Low value.
    /// * `high` - High value.
    fn clamp(&self, low: Float, high: Float) -> Self {
        Self {
            c: [
                clamp(self.c[0], low, high),
                clamp(self.c[1], low, high),
                clamp(self.c[2], low, high),
            ],
        }
    }

    /// Clamps the values to [0.0, INFINITY].
    fn clamp_default(&self) -> Self {
        self.clamp(0.0, INFINITY)
    }
}

impl fmt::Display for RGBSpectrum {
    /// Formats the value using the given formatter.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}, {}]", self.c[0], self.c[1], self.c[2])
    }
}
