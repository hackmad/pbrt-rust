//! Paraboloids

#![allow(dead_code)]
use super::{
    bounds3, clamp, efloat, intersection, max, min, point2, point3, quadratic, shape_data,
    surface_interaction, vector3, ArcTransform, Bounds3f, Dot, EFloat, Float, Intersection,
    Normal3, Ray, Shape, ShapeData, TWO_PI,
};
use std::sync::Arc;

/// A paraboloid centered on the z-axis.
#[derive(Clone)]
pub struct Paraboloid {
    /// Common shape data.
    pub data: ShapeData,

    /// Radius of paraboloid.
    pub radius: Float,

    /// Minimum z-value to truncate paraboloid.
    pub z_min: Float,

    /// Maximum z-value to truncate paraboloid.
    pub z_max: Float,

    /// Maximum angle Φ to truncate paraboloid.
    pub phi_max: Float,
}

/// Create a new paraboloid centered on the z-axis and base at origin [0, 0, 0].
///
/// * `object_to_world`     - The object to world transfomation.
/// * `world_to_object`     - The world to object transfomation.
/// * `reverse_orientation` - Indicates whether their surface normal directions
///                           should be reversed from the default
/// * `radius`              - Radius of paraboloid.
/// * `z_min`               - Minimum z-value to truncate paraboloid.
/// * `z_max`               - Maximum z-value to truncate paraboloid.
pub fn paraboloid(
    object_to_world: ArcTransform,
    world_to_object: ArcTransform,
    reverse_orientation: bool,
    radius: Float,
    z_min: Float,
    z_max: Float,
    phi_max: Float,
) -> Paraboloid {
    let zmin = clamp(min(z_min, z_max), -radius, radius);
    let zmax = clamp(max(z_min, z_max), -radius, radius);
    Paraboloid {
        radius,
        z_min: zmin,
        z_max: zmax,
        phi_max: clamp(phi_max, 0.0, 360.0).to_radians(),
        data: shape_data(
            object_to_world.clone(),
            Some(world_to_object.clone()),
            reverse_orientation,
        ),
    }
}

impl Shape for Paraboloid {
    /// Returns the underlying shape data.
    fn get_data(&self) -> ShapeData {
        self.data.clone()
    }

    /// Returns a bounding box in the shapes object space.
    fn object_bound(&self) -> Bounds3f {
        bounds3(
            point3(-self.radius, -self.radius, self.z_min),
            point3(self.radius, self.radius, self.z_max),
        )
    }

    /// Returns geometric details if a ray intersects the shape intersection.
    /// If there is no intersection, `None` is returned.
    ///
    /// * `r`                  - The ray.
    /// * `test_alpha_texture` - Perform alpha texture tests (not supported).
    fn intersect(&self, r: &Ray, _test_alpha_texture: bool) -> Option<Intersection> {
        // Transform ray to object space
        let (ray, o_err, d_err) = self
            .data
            .world_to_object
            .clone()
            .unwrap()
            .transform_ray_with_error(r);

        // Compute quadratic paraboloid coefficients

        // Initialize EFloat ray coordinate values
        let ox = efloat(ray.o.x, o_err.x);
        let oy = efloat(ray.o.y, o_err.y);
        let oz = efloat(ray.o.z, o_err.z);

        let dx = efloat(ray.d.x, d_err.x);
        let dy = efloat(ray.d.y, d_err.y);
        let dz = efloat(ray.d.z, d_err.z);

        let k = EFloat::from(self.z_max) / (EFloat::from(self.radius) * EFloat::from(self.radius));

        let a = k * (dx * dx + dy * dy);
        let b = 2.0 * k * (dx * ox + dy * oy) - dz;
        let c = k * (ox * ox + oy * oy) - oz;

        // Solve quadratic equation for t values
        if let Some((t0, t1)) = quadratic(a, b, c) {
            // Check quadric shape t0 and t1 for nearest intersection
            if t0.upper_bound() > ray.t_max || t1.lower_bound() <= 0.0 {
                return None;
            }

            let mut t_shape_hit = t0;
            if t_shape_hit.lower_bound() <= 0.0 {
                t_shape_hit = t1;
                if t_shape_hit.upper_bound() > ray.t_max {
                    return None;
                };
            }

            // Compute paraboloid inverse mapping
            let mut p_hit = ray.at(Float::from(t_shape_hit));

            let mut phi = p_hit.y.atan2(p_hit.x);
            if phi < 0.0 {
                phi += TWO_PI;
            }

            // Test paraboloid intersection against clipping parameters
            if p_hit.z < self.z_min || p_hit.z > self.z_max || phi > self.phi_max {
                if t_shape_hit == t1 {
                    return None;
                }

                t_shape_hit = t1;

                if t1.upper_bound() > ray.t_max {
                    return None;
                }

                // Compute paraboloid inverse mapping
                p_hit = ray.at(Float::from(t_shape_hit));

                phi = p_hit.y.atan2(p_hit.x);
                if phi < 0.0 {
                    phi += TWO_PI;
                }

                if p_hit.z < self.z_min || p_hit.z > self.z_max || phi > self.phi_max {
                    return None;
                }
            }

            // Find parametric representation of paraboloid hit
            let u = phi / self.phi_max;
            let v = (p_hit.z - self.z_min) / (self.z_max - self.z_min);

            // Compute paraboloid dpdu and dpdv
            let dpdu = vector3(-self.phi_max * p_hit.y, self.phi_max * p_hit.x, 0.0);
            let dpdv = (self.z_max - self.z_min)
                * vector3(p_hit.x / (2.0 * p_hit.z), p_hit.y / (2.0 * p_hit.z), 1.0);

            // Compute paraboloid dndu and dndv
            let d2p_duu = -self.phi_max * self.phi_max * vector3(p_hit.x, p_hit.y, 0.0);
            let d2p_duv = (self.z_max - self.z_min)
                * self.phi_max
                * vector3(-p_hit.y / (2.0 * p_hit.z), p_hit.x / (2.0 * p_hit.z), 0.0);
            let d2p_dvv = -(self.z_max - self.z_min)
                * (self.z_max - self.z_min)
                * vector3(
                    p_hit.x / (4.0 * p_hit.z * p_hit.z),
                    p_hit.y / (4.0 * p_hit.z * p_hit.z),
                    0.0,
                );

            // Compute normal
            let n = dpdu.cross(&dpdv).normalize();

            // Compute coefficients for first fundamental form
            let e1 = dpdu.dot(&dpdu);
            let f1 = dpdu.dot(&dpdv);
            let g1 = dpdv.dot(&dpdv);

            // Compute coefficients for second fundamental form.
            let e2 = n.dot(&d2p_duu);
            let f2 = n.dot(&d2p_duv);
            let g2 = n.dot(&d2p_dvv);

            // Compute dndu and dndv from fundamental form coefficients
            let inv_egf_1 = 1.0 / (e1 * g1 - f1 * f1);
            let dndu = Normal3::from(
                (f2 * f1 - e2 * g1) * inv_egf_1 * dpdu + (e2 * f1 - f2 * e1) * inv_egf_1 * dpdv,
            );
            let dndv = Normal3::from(
                (g2 * f1 - f2 * g1) * inv_egf_1 * dpdu + (f2 * f1 - g2 * e1) * inv_egf_1 * dpdv,
            );

            // Compute error bounds for paraboloid intersection
            // Compute error bounds for intersection computed with ray equation
            let px = ox + t_shape_hit * dx;
            let py = oy + t_shape_hit * dy;
            let pz = oz + t_shape_hit * dz;
            let p_error = vector3(
                px.get_absolute_error(),
                py.get_absolute_error(),
                pz.get_absolute_error(),
            );

            // Initialize SurfaceInteraction from parametric information
            let si = surface_interaction(
                p_hit,
                p_error,
                point2(u, v),
                -ray.d,
                dpdu,
                dpdv,
                dndu,
                dndv,
                ray.time,
                Some(Arc::new(self.clone())),
            );

            // Create hit.
            let isect = self.data.object_to_world.transform_surface_interaction(&si);
            let t_hit = Float::from(t_shape_hit);
            Some(intersection(t_hit, isect))
        } else {
            None
        }
    }

    /// Returns `true` if a ray-shape intersection succeeds; otherwise `false`.
    ///
    /// * `r`                  - The ray.
    /// * `test_alpha_texture` - Perform alpha texture tests (not supported).
    fn intersect_p(&self, r: &Ray, _test_alpha_texture: bool) -> bool {
        // Transform ray to object space
        let (ray, o_err, d_err) = self
            .data
            .world_to_object
            .clone()
            .unwrap()
            .transform_ray_with_error(r);

        // Compute quadratic paraboloid coefficients

        // Initialize EFloat ray coordinate values
        let ox = efloat(ray.o.x, o_err.x);
        let oy = efloat(ray.o.y, o_err.y);
        let oz = efloat(ray.o.z, o_err.z);

        let dx = efloat(ray.d.x, d_err.x);
        let dy = efloat(ray.d.y, d_err.y);
        let dz = efloat(ray.d.z, d_err.z);

        let k = EFloat::from(self.z_max) / (EFloat::from(self.radius) * EFloat::from(self.radius));

        let a = k * (dx * dx + dy * dy);
        let b = 2.0 * k * (dx * ox + dy * oy) - dz;
        let c = k * (ox * ox + oy * oy) - oz;

        // Solve quadratic equation for t values
        if let Some((t0, t1)) = quadratic(a, b, c) {
            // Check quadric shape t0 and t1 for nearest intersection
            if t0.upper_bound() > ray.t_max || t1.lower_bound() <= 0.0 {
                return false;
            }

            let mut t_shape_hit = t0;
            if t_shape_hit.lower_bound() <= 0.0 {
                t_shape_hit = t1;
                if t_shape_hit.upper_bound() > ray.t_max {
                    return false;
                };
            }

            // Compute paraboloid inverse mapping
            let mut p_hit = ray.at(Float::from(t_shape_hit));

            let mut phi = p_hit.y.atan2(p_hit.x);
            if phi < 0.0 {
                phi += TWO_PI;
            }

            // Test paraboloid intersection against clipping parameters
            if p_hit.z < self.z_min || p_hit.z > self.z_max || phi > self.phi_max {
                if t_shape_hit == t1 {
                    return false;
                }

                t_shape_hit = t1;

                if t1.upper_bound() > ray.t_max {
                    return false;
                }

                // Compute paraboloid inverse mapping
                p_hit = ray.at(Float::from(t_shape_hit));

                phi = p_hit.y.atan2(p_hit.x);
                if phi < 0.0 {
                    phi += TWO_PI;
                }

                if p_hit.z < self.z_min || p_hit.z > self.z_max || phi > self.phi_max {
                    return false;
                }
            }
        } else {
            return false;
        }

        true
    }

    /// Returns the surface area of the shape in object space.
    fn area(&self) -> Float {
        let radius2 = self.radius * self.radius;
        let k = 4.0 * self.z_max / radius2;
        (radius2 * radius2 * self.phi_max / (12.0 * self.z_max * self.z_max))
            * ((k * self.z_max + 1.0).powf(1.5) - (k * self.z_min + 1.0).powf(1.5))
    }
}