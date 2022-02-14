//! Tabulated BSSRDF

#![allow(dead_code)]

use super::*;
use crate::bssrdf::*;
use crate::interaction::*;
use crate::interpolation::*;
use crate::material::*;
use crate::medium::phase_hg;
use crate::scene::*;
use bumpalo::collections::vec::Vec as BumpVec;
use bumpalo::Bump;

/// A tabulated BSSRDF representation that can handle a wide range of scattering
/// profiles including measured real-world BSSRDFs.
pub struct TabulatedBSSRDF<'arena> {
    /// BxDF type.
    bxdf_type: BxDFType,

    /// Scattering profile details.
    table: &'arena mut BSSRDFTable<'arena>,

    /// Total reduction in radiance due to absorption and out-scattering
    /// `σt = σs + σa`. This combined effect of absorption and out-scattering is
    /// called attenuation or extinction./
    sigma_t: Spectrum,

    /// Albedo.
    rho: Spectrum,

    /// Separable BSSRDF.
    bssrdf: &'arena mut SeparableBSSRDF,
}

impl<'arena> TabulatedBSSRDF<'arena> {
    /// Allocate a new instance of `TabulatedBSSRDF`.
    ///
    /// * `arena`    - The arena for memory allocations.
    /// * `po`       - Current outgoing surface interaction.
    /// * `eta`      - Index of refraction of the scattering medium.
    /// * `material` - The material.
    /// * `mode`     - Light transport mode.
    /// * `sigma_a`  - Absorption coefficient `σa`.
    /// * `sigma_s`  - Scattering coefficient `σs`.
    /// * `table`    - Detailed BSSRDF information.
    #[allow(clippy::mut_from_ref)]
    pub fn alloc(
        arena: &'arena Bump,
        po: &SurfaceInteraction,
        eta: Float,
        material: ArcMaterial,
        mode: TransportMode,
        sigma_a: Spectrum,
        sigma_s: Spectrum,
        table: &'arena mut BSSRDFTable<'arena>,
    ) -> &'arena mut BxDF<'arena> {
        let sigma_t = sigma_a + sigma_s;

        let mut rho = Spectrum::ZERO;
        for c in 0..SPECTRUM_SAMPLES {
            rho[c] = if sigma_t[c] != 0.0 {
                sigma_s[c] / sigma_t[c]
            } else {
                0.0
            };
        }

        let bssrdf = SeparableBSSRDF::alloc(arena, po, eta, material, mode);

        let model = arena.alloc(Self {
            bxdf_type: BxDFType::BSDF_REFLECTION | BxDFType::BSDF_DIFFUSE,
            table,
            sigma_t,
            rho,
            bssrdf,
        });
        arena.alloc(BxDF::TabulatedBSSRDF(model))
    }

    /// Clone into a newly allocated a new instance of `TabulatedBSSRDF`.
    ///
    /// * `arena` - The arena for memory allocations.
    #[allow(clippy::mut_from_ref)]
    pub fn clone_alloc(&self, arena: &'arena Bump) -> &'arena mut BxDF<'arena> {
        let table = self.table.clone_alloc(arena);
        let bssrdf = self.bssrdf.clone_alloc(arena);
        let model = arena.alloc(Self {
            bxdf_type: self.bxdf_type,
            table,
            sigma_t: self.sigma_t,
            rho: self.rho,
            bssrdf,
        });
        arena.alloc(BxDF::TabulatedBSSRDF(model))
    }

    /// Returns the BxDF type.
    pub fn get_type(&self) -> BxDFType {
        self.bxdf_type
    }

    /// Returns the value of the distribution function for the given pair of
    /// directions.
    ///
    /// * `wo` - Outgoing direction.
    /// * `wi` - Incident direction.
    pub fn f(&self, _wo: &Vector3f, wi: &Vector3f) -> Spectrum {
        let f = self.bssrdf.sw(wi);

        // Update BSSRDF transmission term to account for adjoint light
        // transport.
        if self.bssrdf.mode == TransportMode::Radiance {
            f * self.bssrdf.eta * self.bssrdf.eta
        } else {
            f
        }
    }

    /// Evaluates the eight-dimensional distribution function S(), which quantifies / the ratio of differential radiance at point `po` in direction `ωo` to the
    /// incident differential flux at `pi` from direction `ωi`.
    ///
    /// * `pi` - Interaction point for incident differential flux.
    /// * `wi` - Direction for incident different flux.
    /// * `sr` - Evaluates the radial profile function based on distance between points.
    pub fn s<Sr>(&self, pi: &SurfaceInteraction, wi: &Vector3f, sr: Sr) -> Spectrum
    where
        Sr: Fn(Float) -> Spectrum,
    {
        self.bssrdf.s(pi, wi, sr)
    }

    /// Evaluates the spatial term for the distribution function.
    ///
    /// * `pi` - Interaction point for incident differential flux.
    /// * `sr` - Evaluates the radial profile function based on distance between points.
    pub fn sp<Sr>(&self, pi: &SurfaceInteraction, sr: Sr) -> Spectrum
    where
        Sr: Fn(Float) -> Spectrum,
    {
        self.bssrdf.sp(pi, sr)
    }

    /// Evaluates the directional term for the distribution function.
    ///
    /// * `w` - Direction for incident different flux.
    pub fn sw(&self, w: &Vector3f) -> Spectrum {
        self.bssrdf.sw(w)
    }

    /// Evaluates the radial profile function based on distance between points.
    ///
    /// * `r` - Distance between points.
    pub fn sr(&self, r: Float) -> Spectrum {
        todo!()
    }

    /// Returns the value of the BSSRDF, the surface position where a ray
    /// re-emerges following internal scattering and probability density function.
    ///
    /// * `arena` - The arena for memory allocations.
    /// * `scene` - The scene.
    /// * `u1`    - Sample values for Monte Carlo.
    /// * `u2`    - Sample values for Monte Carlo.
    /// * `si`    - The surface position where a ray re-emerges following internal
    ///             scattering.
    pub fn sample_s(
        &self,
        arena: &'arena Bump,
        scene: &Scene,
        u1: Float,
        u2: &Point2f,
        si: &mut SurfaceInteraction<'_, 'arena>,
    ) -> (Spectrum, Float) {
        self.bssrdf.sample_s(
            arena,
            scene,
            u1,
            u2,
            si,
            |ch, u| self.sample_sr(ch, u),
            |si| self.pdf_sp(si),
            |r| self.sr(r),
        )
    }

    /// Use a different sampling technique per wavelength to deal with spectral
    /// variation, and each technique is additionally replicated three times with
    /// different projection axes given by the basis vectors of a local frame,
    /// resulting in a total of 3 * Spectrum::nSamples sampling techniques. This
    /// ensures that every point `S` where takes on non-negligible values is
    /// intersected with a reasonable probability.
    ///
    /// * `scene` - The scene.
    /// * `u1`    - Sample values for Monte Carlo.
    /// * `u2`    - Sample values for Monte Carlo.
    /// * `si`    - Surface interaction.
    pub fn sample_sp<'scene>(
        &self,
        scene: &'scene Scene,
        u1: Float,
        u2: &Point2f,
        si: &'scene mut SurfaceInteraction,
    ) -> (Spectrum, Float) {
        self.bssrdf.sample_sp(
            scene,
            u1,
            u2,
            si,
            |ch, u| self.sample_sr(ch, u),
            |si| self.pdf_sp(si),
            |r| self.sr(r),
        )
    }

    /// Evaluate the combined PDF that takes all of the sampling strategies
    /// `sample_sp()` into account.
    ///
    /// * `si` - Surface interaction.
    pub fn pdf_sp(&self, si: &SurfaceInteraction) -> Float {
        self.bssrdf.pdf_sp(si)
    }

    /// Samples radius values proportional to the radial profile function.
    ///
    /// * `ch` - Channel.
    /// * `u`  - Sample value.
    pub fn sample_sr(&self, ch: usize, u: Float) -> Float {
        if self.sigma_t[ch] == 0.0 {
            -1.0
        } else {
            let (sample, _, _) = sample_catmull_rom_2d(
                &self.table.rho_samples,
                &self.table.radius_samples,
                &self.table.profile,
                &self.table.profile_cdf,
                self.rho[ch],
                u,
            );
            sample / self.sigma_t[ch]
        }
    }

    /// Returns the PDF of samples obtained via `sample_sr()`. It evaluates the
    /// profile function divided by the normalizing constant.
    ///
    /// * `ch` - Channel.
    /// * `r`  - Radius.
    pub fn pdf_sr(&self, ch: usize, r: Float) -> Float {
        // Convert `r` into unitless optical radius `r_optical`.
        let r_optical = r * self.sigma_t[ch];

        // Compute spline weights to interpolate BSSRDF density on channel `ch`.
        let rho = catmull_rom_weights(&self.table.rho_samples, self.rho[ch]);
        let radius = catmull_rom_weights(&self.table.radius_samples, r_optical);
        if rho.is_none() || radius.is_none() {
            return 0.0;
        }
        let (rho_weights, rho_offset) = rho.unwrap();
        let (radius_weights, radius_offset) = radius.unwrap();

        // Return BSSRDF profile density for channel `ch`.
        let mut sr = 0.0;
        let mut rho_eff = 0.0;
        for i in 0..4 {
            if rho_weights[i] == 0.0 {
                continue;
            }
            rho_eff += self.table.rho_eff[rho_offset + i] * rho_weights[i];
            for j in 0..4 {
                if radius_weights[j] == 0.0 {
                    continue;
                }
                sr += self.table.eval_profile(rho_offset + i, radius_offset + j)
                    * rho_weights[i]
                    * radius_weights[j];
            }
        }

        // Cancel marginal PDF factor from tabulated BSSRDF profile.
        if r_optical != 0.0 {
            sr /= TWO_PI * r_optical;
        }
        max(0.0, sr * self.sigma_t[ch] * self.sigma_t[ch] / rho_eff)
    }
}

/// Stores detailed information about the scattering profile `Sr`.
pub struct BSSRDFTable<'arena> {
    /// Single scattering albedos.
    rho_samples: BumpVec<'arena, Float>,

    /// Radii samples.
    radius_samples: BumpVec<'arena, Float>,

    /// Sample values for each of the n_rho_samples X n_radius_samples.
    profile: BumpVec<'arena, Float>,

    /// CDF for the sample values `profile`.
    profile_cdf: BumpVec<'arena, Float>,

    /// Effective albedo.
    rho_eff: BumpVec<'arena, Float>,

    /// Number of single scattering albedos.
    n_rho_samples: usize,

    /// Number of radii albedos.
    n_radius_samples: usize,
}

impl<'arena> BSSRDFTable<'arena> {
    /// Allocate a new instance of `BSSRDFTable`.
    ///
    /// * `arena`            - The arena for memory allocations.
    /// * `n_rho_samples`    - Number of single scattering albedos.
    /// * `n_radius_samples` - Number of radii albedos.
    pub fn alloc(
        arena: &'arena Bump,
        n_rho_samples: usize,
        n_radius_samples: usize,
    ) -> &'arena mut Self {
        let rho_samples = BumpVec::with_capacity_in(n_rho_samples, arena);
        let radius_samples = BumpVec::with_capacity_in(n_radius_samples, arena);
        let profile = BumpVec::with_capacity_in(n_radius_samples * n_rho_samples, arena);
        let profile_cdf = BumpVec::with_capacity_in(n_radius_samples * n_rho_samples, arena);
        let rho_eff = BumpVec::with_capacity_in(n_rho_samples, arena);

        arena.alloc(Self {
            rho_samples,
            radius_samples,
            profile,
            rho_eff,
            profile_cdf,
            n_rho_samples,
            n_radius_samples,
        })
    }

    /// Clone into a newly allocated a new instance of `TabulatedBSSRDF`.
    ///
    /// * `arena` - The arena for memory allocations.
    #[allow(clippy::mut_from_ref)]
    pub fn clone_alloc(&self, arena: &'arena Bump) -> &'arena mut Self {
        let mut rho_samples = BumpVec::with_capacity_in(self.n_rho_samples, arena);
        let mut radius_samples = BumpVec::with_capacity_in(self.n_radius_samples, arena);
        let mut profile =
            BumpVec::with_capacity_in(self.n_radius_samples * self.n_rho_samples, arena);
        let mut profile_cdf =
            BumpVec::with_capacity_in(self.n_radius_samples * self.n_rho_samples, arena);
        let mut rho_eff = BumpVec::with_capacity_in(self.n_rho_samples, arena);

        for i in 0..self.n_rho_samples {
            rho_samples[i] = self.rho_samples[i];
        }
        for i in 0..self.n_radius_samples {
            radius_samples[i] = self.radius_samples[i];
        }
        for i in 0..self.n_radius_samples * self.n_rho_samples {
            profile[i] = self.profile[i];
            profile_cdf[i] = self.profile_cdf[i];
        }
        for i in 0..self.n_rho_samples {
            rho_eff[i] = self.rho_eff[i];
        }

        arena.alloc(Self {
            rho_samples,
            radius_samples,
            profile,
            rho_eff,
            profile_cdf,
            n_rho_samples: self.n_rho_samples,
            n_radius_samples: self.n_radius_samples,
        })
    }

    /// Finds profile values for a given albedo and radius sample index.
    ///
    /// * `rho_index`    - Index into the albedo samples.
    /// * `radius_index` - Index into the radius samples.
    pub fn eval_profile(&self, rho_index: usize, radius_index: usize) -> Float {
        self.profile[rho_index * self.radius_samples.len() + radius_index]
    }

    /// Returns a medium's scattering properties; absorption coefficient `σa` and
    /// scattering coefficient `σs`.
    ///
    /// * `rho_eff` - Effective albedo.
    /// * `mfp`     - Mean free path.
    pub fn subsurface_from_diffuse(
        &self,
        rho_eff: &Spectrum,
        mfp: &Spectrum,
    ) -> (Spectrum, Spectrum) {
        let mut sigma_a = Spectrum::ZERO;
        let mut sigma_s = Spectrum::ZERO;

        for c in 0..SPECTRUM_SAMPLES {
            let rho = invert_catmull_rom(&self.rho_samples, &self.rho_eff, rho_eff[c]);
            sigma_s[c] = rho / mfp[c];
            sigma_a[c] = (1.0 - rho) / mfp[c];
        }

        (sigma_a, sigma_s)
    }

    /// Fill the profile data using the photon beam diffusion functions.
    ///
    /// * `g`   - The asymmetry parameter for Henyey-Greenstein phase function.
    /// * `eta` - Index of refraction of the scattering medium.
    pub fn compute_beam_diffusion(&mut self, g: Float, eta: Float) {
        // Choose radius values of the diffusion profile discretization.
        self.radius_samples[0] = 0.0;
        self.radius_samples[1] = 2.5e-3;
        for i in 2..self.n_radius_samples {
            self.radius_samples[i] = self.radius_samples[i - 1] * 1.2;
        }

        // Choose albedo values of the diffusion profile discretization.
        for (i, rho_sample) in self.rho_samples.iter_mut().enumerate() {
            *rho_sample = (1.0 - (-8.0 * i as Float / (self.n_rho_samples - 1) as Float).exp())
                / (1.0 - (-8.0 as Float).exp());
        }

        self.rho_samples.iter().enumerate().for_each(|(i, &rho)| {
            // Compute the diffusion profile for the i^th albedo sample.
            //
            // Compute scattering profile for chosen albedo `rho`.
            for j in 0..self.n_radius_samples {
                let r = self.radius_samples[j];
                self.profile[i * self.n_radius_samples + j] = TWO_PI
                    * r
                    * (beam_diffusion_ss(rho, 1.0 - rho, g, eta, r)
                        + beam_diffusion_ms(rho, 1.0 - rho, g, eta, r));
            }

            // Compute effective albedo `rho_eff` and CDF for importance sampling.
            let (profile_cdf, rho_eff) = integrate_catmull_rom(
                &self.radius_samples,
                &self.profile[i * self.n_radius_samples..],
            );
            self.rho_eff[i] = rho_eff;
            self.profile_cdf[i * self.n_radius_samples..].copy_from_slice(&profile_cdf);
        });
    }
}

/// Number of samples to use for the photon beam diffusion integral estimates.
const PBD_SAMPLES: usize = 100;

/// Compute the photon beam diffusion (PBD) single-scattering profile using 100
/// samples for the integral estimate.
///
/// * `sigma_s` - Scattering coefficient `σs` is the probability of an out-scattering
///               event occurring per unit distance
/// * `sigma_a` - Absorption cross section `σa` is the probability density that
///               light is absorbed per unit distance traveled in the medium
/// * `g`       - The asymmetry parameter for Henyey-Greenstein phase function.
/// * `eta`     - Index of refraction of the scattering medium.
/// * `radius`  - Radius.
#[allow(non_snake_case)]
fn beam_diffusion_ss(sigma_s: Float, sigma_a: Float, g: Float, eta: Float, r: Float) -> Float {
    // Compute material parameters and minimum $t$ below the critical angle.
    let sigma_t = sigma_a + sigma_s;
    let rho = sigma_s / sigma_t;
    let tCrit = r * (eta * eta - 1.0).sqrt();
    let mut Ess = 0.0;

    for i in 0..PBD_SAMPLES {
        // Evaluate single scattering integrand and add to `Ess`.
        let ti = tCrit - (1.0 - (i as Float + 0.5) / PBD_SAMPLES as Float).ln() / sigma_t;

        // Determine length $d$ of connecting segment and $\cos\theta_\roman{o}$
        let d = (r * r + ti * ti).sqrt();
        let cosThetaO = ti / d;

        // Add contribution of single scattering at depth `t`.
        Ess += rho * (-sigma_t * (d + tCrit)).exp() / (d * d)
            * phase_hg(cosThetaO, g)
            * (1.0 - fr_dielectric(-cosThetaO, 1.0, eta))
            * abs(cosThetaO);
    }
    Ess / PBD_SAMPLES as Float
}

/// Compute the average of 100 samples of the photon beam diffusion (PBD) integrand.
///
/// * `sigma_s` - Scattering coefficient `σs` is the probability of an out-scattering
///               event occurring per unit distance
/// * `sigma_a` - Absorption cross section `σa` is the probability density that
///               light is absorbed per unit distance traveled in the medium
/// * `g`       - The asymmetry parameter for Henyey-Greenstein phase function.
/// * `eta`     - Index of refraction of the scattering medium.
/// * `radius`  - Radius.
#[allow(non_snake_case)]
fn beam_diffusion_ms(sigma_s: Float, sigma_a: Float, g: Float, eta: Float, r: Float) -> Float {
    let mut Ed = 0.0;

    // Precompute information for dipole integrand.

    // Compute reduced scattering coefficients `sigmap_s`, `sigmap_t` and albedo `rhop`.
    let sigmap_s = sigma_s * (1.0 - g);
    let sigmap_t = sigma_a + sigmap_s;
    let rhop = sigmap_s / sigmap_t;

    // Compute non-classical diffusion coefficient `D_G` using Equation (15.24).
    let D_g = (2.0 * sigma_a + sigmap_s) / (3.0 * sigmap_t * sigmap_t);

    // Compute effective transport coefficient `sigma_tr` based on `D_G`.
    let sigma_tr = (sigma_a / D_g).sqrt();

    // Determine linear extrapolation distance depth using Equation (15.28).
    let fm1 = fresnel_moment_1(eta);
    let fm2 = fresnel_moment_2(eta);
    let ze = -2.0 * D_g * (1.0 + 3.0 * fm2) / (1.0 - 2.0 * fm1);

    // Determine exitance scale factors using Equations (15.31) and (15.32).
    let cPhi = 0.25 * (1.0 - 2.0 * fm1);
    let cE = 0.5 * (1.0 - 3.0 * fm2);
    for i in 0..PBD_SAMPLES {
        // Sample real point source depth `depth_real`.
        let zr = -(1.0 - (i as Float + 0.5) / PBD_SAMPLES as Float).ln() / sigmap_t;

        // Evaluate dipole integrand `E_d` at `depth_real` and add to `Ed`.
        let zv = -zr + 2.0 * ze;
        let dr = (r * r + zr * zr).sqrt();
        let dv = (r * r + zv * zv).sqrt();

        // Compute dipole fluence rate using Equation (15.27).
        let phiD = INV_FOUR_PI / D_g * ((-sigma_tr * dr).exp() / dr - (-sigma_tr * dv).exp() / dv);

        // Compute dipole vector irradiance `n dot Ed(r)` using Equation (15.27).
        let EDn = INV_FOUR_PI
            * (zr * (1.0 + sigma_tr * dr) * (-sigma_tr * dr).exp() / (dr * dr * dr)
                - zv * (1.0 + sigma_tr * dv) * (-sigma_tr * dv).exp() / (dv * dv * dv));

        // Add contribution from dipole for depth `depth_real` to `Ed`.
        let E = phiD * cPhi + EDn * cE;
        let kappa = 1.0 - (-2.0 * sigmap_t * (dr + zr)).exp();
        Ed += kappa * rhop * rhop * E;
    }

    Ed / PBD_SAMPLES as Float
}