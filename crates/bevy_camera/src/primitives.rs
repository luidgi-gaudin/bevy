use core::borrow::Borrow;

use bevy_derive::{Deref, DerefMut};
use bevy_ecs::{component::Component, entity::EntityHashMap, reflect::ReflectComponent};
use bevy_math::{Affine3A, Mat3A, Vec3, Vec3A};
use bevy_mesh::{Mesh, VertexAttributeValues};
use bevy_reflect::prelude::*;
use bevy_shape::{Aabb3d, BoundingVolume, HalfSpace, ViewFrustum};

pub trait MeshAabb {
    /// Compute the Axis-Aligned Bounding Box of the mesh vertices in model space,
    /// or return [`Mesh::final_aabb`] if it is not `None`.
    ///
    /// Returns `None` if `self` doesn't have [`Mesh::ATTRIBUTE_POSITION`] of
    /// type [`VertexAttributeValues::Float32x3`], or if `self` doesn't have any vertices.
    fn get_aabb(&self) -> Option<Aabb>;
}

impl MeshAabb for Mesh {
    fn get_aabb(&self) -> Option<Aabb> {
        if let Some(aabb) = self.final_aabb {
            // use precomputed extents
            return Some(aabb.into());
        }

        let Ok(VertexAttributeValues::Float32x3(values)) =
            self.try_attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            return None;
        };

        Aabb::enclosing(values.iter().map(|p| Vec3::from_slice(p)))
    }
}

/// An axis-aligned bounding box, defined by:
/// - a center,
/// - the distances from the center to each faces along the axis,
///   the faces are orthogonal to the axis.
///
/// It is typically used as a component on an entity to represent the local space
/// occupied by this entity, with faces orthogonal to its local axis.
///
/// This component is notably used during "frustum culling", a process to determine
/// if an entity should be rendered by a [`Camera`] if its bounding box intersects
/// with the camera's [`Frustum`].
///
/// It will be added automatically by the systems in [`CalculateBounds`] to entities that:
/// - could be subject to frustum culling, for example with a [`Mesh3d`]
///   or `Sprite` component,
/// - don't have the [`NoFrustumCulling`] component.
///
/// It won't be updated automatically if the space occupied by the entity changes,
/// for example if the vertex positions of a [`Mesh3d`] are updated.
///
/// [`Camera`]: crate::Camera
/// [`NoFrustumCulling`]: crate::visibility::NoFrustumCulling
/// [`CalculateBounds`]: crate::visibility::VisibilitySystems::CalculateBounds
/// [`Mesh3d`]: bevy_mesh::Mesh
#[derive(Component, Clone, Copy, Debug, Default, Reflect, PartialEq)]
#[reflect(Component, Default, Debug, PartialEq, Clone)]
pub struct Aabb {
    pub center: Vec3A,
    pub half_extents: Vec3A,
}

impl Aabb {
    #[inline]
    pub fn from_min_max(minimum: Vec3, maximum: Vec3) -> Self {
        let minimum = Vec3A::from(minimum);
        let maximum = Vec3A::from(maximum);
        let center = 0.5 * (maximum + minimum);
        let half_extents = 0.5 * (maximum - minimum);
        Self {
            center,
            half_extents,
        }
    }

    /// Returns a bounding box enclosing the specified set of points.
    ///
    /// Returns `None` if the iterator is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bevy_math::{Vec3, Vec3A};
    /// # use bevy_camera::primitives::Aabb;
    /// let bb = Aabb::enclosing([Vec3::X, Vec3::Z * 2.0, Vec3::Y * -0.5]).unwrap();
    /// assert_eq!(bb.min(), Vec3A::new(0.0, -0.5, 0.0));
    /// assert_eq!(bb.max(), Vec3A::new(1.0, 0.0, 2.0));
    /// ```
    pub fn enclosing<T: Borrow<Vec3>>(iter: impl IntoIterator<Item = T>) -> Option<Self> {
        let mut iter = iter.into_iter().map(|p| *p.borrow());
        let mut min = iter.next()?;
        let mut max = min;
        for v in iter {
            min = Vec3::min(min, v);
            max = Vec3::max(max, v);
        }
        Some(Self::from_min_max(min, max))
    }

    /// Calculate the relative radius of the AABB with respect to a plane
    #[inline]
    pub fn relative_radius(&self, p_normal: &Vec3A, world_from_local: &Mat3A) -> f32 {
        // NOTE: dot products on Vec3A use SIMD and even with the overhead of conversion are net faster than Vec3
        let half_extents = self.half_extents;
        Vec3A::new(
            p_normal.dot(world_from_local.x_axis),
            p_normal.dot(world_from_local.y_axis),
            p_normal.dot(world_from_local.z_axis),
        )
        .abs()
        .dot(half_extents)
    }

    /// Calculate the radius of a sphere centered on the AABB that contains the whole AABB once
    /// transformed by `world_from_local`, even when the transform has shear.
    #[inline]
    pub fn bounding_sphere_radius(&self, world_from_local: &Mat3A) -> f32 {
        Vec3A::new(
            world_from_local.x_axis.length(),
            world_from_local.y_axis.length(),
            world_from_local.z_axis.length(),
        )
        .dot(self.half_extents)
    }

    #[inline]
    pub fn min(&self) -> Vec3A {
        self.center - self.half_extents
    }

    #[inline]
    pub fn max(&self) -> Vec3A {
        self.center + self.half_extents
    }

    /// Check if the AABB is at the front side of the bisecting plane.
    /// Referenced from: [AABB Plane intersection](https://gdbooks.gitbooks.io/3dcollisions/content/Chapter2/static_aabb_plane.html)
    #[inline]
    pub fn is_in_half_space(&self, half_space: &HalfSpace, world_from_local: &Affine3A) -> bool {
        // transform the half-extents into world space.
        let half_extents_world = world_from_local.matrix3.abs() * self.half_extents.abs();
        // collapse the half-extents onto the plane normal.
        let p_normal = half_space.normal();
        let r = half_extents_world.dot(p_normal.abs());
        let aabb_center_world = world_from_local.transform_point3a(self.center);
        let signed_distance = p_normal.dot(aabb_center_world) + half_space.d();
        signed_distance > r
    }

    /// Optimized version of [`Self::is_in_half_space`] when the AABB is already in world space.
    /// Use this when `world_from_local` would be the identity transform.
    #[inline]
    pub fn is_in_half_space_identity(&self, half_space: &HalfSpace) -> bool {
        let p_normal = half_space.normal();
        let r = self.half_extents.abs().dot(p_normal.abs());
        let signed_distance = p_normal.dot(self.center) + half_space.d();
        signed_distance > r
    }
}

impl From<Aabb3d> for Aabb {
    fn from(aabb: Aabb3d) -> Self {
        Self {
            center: aabb.center(),
            half_extents: aabb.half_size(),
        }
    }
}

impl From<Aabb> for Aabb3d {
    fn from(aabb: Aabb) -> Self {
        Self {
            min: aabb.min(),
            max: aabb.max(),
        }
    }
}

impl From<Sphere> for Aabb {
    #[inline]
    fn from(sphere: Sphere) -> Self {
        Self {
            center: sphere.center,
            half_extents: Vec3A::splat(sphere.radius),
        }
    }
}

/// A sphere, defined by a center and a radius.
///
/// This is typically used as a component on an entity to represent the local
/// space occupied by this entity, as an alternative to [`Aabb`]. The *frustum
/// culling* process uses this component to determine whether an entity is in
/// the view of a [`crate::Camera`].
///
/// Bevy will automatically add this component to point and spot lights, as
/// their ranges are most easily approximated by a sphere. The engine will keep
/// this entity updated as the range and/or transform of such lights changes.
///
/// If both [`Aabb`] and [`Sphere`] are present on an entity, [`Aabb`] takes
/// precedence.
#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component, Clone, Debug, Default)]
pub struct Sphere {
    /// The center of the sphere.
    ///
    /// If this is used as a component, [`Self::center`] is in local space. That
    /// is, it doesn't take the world-space position of the object into account.
    pub center: Vec3A,

    /// The radius of the sphere.
    ///
    /// If this is used as a component, [`Self::radius`] is in local space. That
    /// is, it doesn't take the world-space scale of the object into account.
    pub radius: f32,
}

impl Sphere {
    /// Returns true if this sphere intersects the given oriented bounding box.
    ///
    /// The oriented bounding box (OBB) to test against is produced by
    /// transforming the given AABB according to the supplied matrix.
    #[inline]
    pub fn intersects_obb(&self, aabb: &Aabb, world_from_local: &Affine3A) -> bool {
        let aabb_center_world = world_from_local.transform_point3a(aabb.center);
        let v = aabb_center_world - self.center;
        let d_sq = v.length_squared();
        let d = d_sq.sqrt();
        let relative_radius_unscaled = aabb.relative_radius(&v, &world_from_local.matrix3);
        d_sq <= self.radius * d + relative_radius_unscaled
    }
}

/// A frustum component is used on an entity with a [`Camera`] component to
/// determine which entities will be considered for rendering by this camera.
/// All entities with an [`Aabb`] component that are not contained by (or crossing
/// the boundary of) the frustum will not be rendered, and not be used in rendering computations.
///
/// This process is called frustum culling, and entities can opt out of it using
/// the [`NoFrustumCulling`] component.
///
/// The frustum component is typically added automatically for cameras, either [`Camera2d`] or [`Camera3d`].
/// It is usually updated automatically by [`update_frusta`] from the
/// [`CameraProjection`] component and [`GlobalTransform`] of the camera entity.
///
/// [`Camera`]: crate::Camera
/// [`NoFrustumCulling`]: crate::visibility::NoFrustumCulling
/// [`update_frusta`]: crate::visibility::update_frusta
/// [`CameraProjection`]: crate::CameraProjection
/// [`GlobalTransform`]: bevy_transform::components::GlobalTransform
/// [`Camera2d`]: crate::Camera2d
/// [`Camera3d`]: crate::Camera3d
#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component, Default, Debug, Clone)]
pub struct Frustum(pub ViewFrustum);

impl Frustum {
    /// Checks if a sphere intersects the frustum.
    #[inline]
    pub fn intersects_sphere(&self, sphere: &Sphere, intersect_far: bool) -> bool {
        let sphere_center = sphere.center.extend(1.0);
        let max = if intersect_far {
            ViewFrustum::FAR_PLANE_IDX
        } else {
            ViewFrustum::NEAR_PLANE_IDX
        };
        for half_space in &self.half_spaces[..=max] {
            if half_space.normal_d().dot(sphere_center) + sphere.radius <= 0.0 {
                return false;
            }
        }
        true
    }

    /// Checks if an Oriented Bounding Box (obb) intersects the frustum.
    #[inline]
    pub fn intersects_obb(
        &self,
        aabb: &Aabb,
        world_from_local: &Affine3A,
        intersect_near: bool,
        intersect_far: bool,
    ) -> bool {
        let aabb_center_world = world_from_local.transform_point3a(aabb.center).extend(1.0);

        for (idx, half_space) in self.half_spaces.into_iter().enumerate() {
            if (idx == ViewFrustum::NEAR_PLANE_IDX && !intersect_near)
                || (idx == ViewFrustum::FAR_PLANE_IDX && !intersect_far)
            {
                continue;
            }
            let p_normal = half_space.normal();
            let relative_radius = aabb.relative_radius(&p_normal, &world_from_local.matrix3);
            if half_space.normal_d().dot(aabb_center_world) + relative_radius <= 0.0 {
                return false;
            }
        }
        true
    }

    /// Checks if an Oriented Bounding Box (obb) intersects the frustum, testing its bounding sphere
    /// first.
    ///
    /// This returns the same result as [`Frustum::intersects_sphere`] for the sphere of radius
    /// `bounding_radius` centered on the obb, followed by [`Frustum::intersects_obb`], both
    /// against the same planes. It is faster than calling them one after the other: the center of
    /// the obb is only transformed once, and the obb is only tested against the planes its center
    /// is outside of, since it can't be outside of the others. For an obb whose center is inside
    /// the frustum, the common case for visible entities, that is two dot products per plane
    /// instead of six.
    ///
    /// When the sphere contains the whole obb, as with [`Aabb::bounding_sphere_radius`], the
    /// result is the same as [`Frustum::intersects_obb`] alone.
    #[inline]
    pub fn intersects_obb_with_bounding_sphere(
        &self,
        aabb: &Aabb,
        world_from_local: &Affine3A,
        bounding_radius: f32,
        intersect_near: bool,
        intersect_far: bool,
    ) -> bool {
        let aabb_center_world = world_from_local.transform_point3a(aabb.center).extend(1.0);
        let max = if intersect_far {
            ViewFrustum::FAR_PLANE_IDX
        } else {
            ViewFrustum::NEAR_PLANE_IDX
        };
        let half_spaces = &self.half_spaces[..=max];
        let skip_near = |idx| idx == ViewFrustum::NEAR_PLANE_IDX && !intersect_near;

        // First, the bounding sphere: a single dot product per plane, which is enough to reject
        // most of the obbs outside of the frustum.
        for (idx, half_space) in half_spaces.iter().enumerate() {
            if !skip_near(idx)
                && half_space.normal_d().dot(aabb_center_world) + bounding_radius <= 0.0
            {
                return false;
            }
        }

        // Then the obb, but only against the planes its center is outside of: it can't be
        // outside of the others.
        for (idx, half_space) in half_spaces.iter().enumerate() {
            let distance = half_space.normal_d().dot(aabb_center_world);
            if !skip_near(idx)
                && distance <= 0.0
                && distance + aabb.relative_radius(&half_space.normal(), &world_from_local.matrix3)
                    <= 0.0
            {
                return false;
            }
        }
        true
    }

    /// Optimized version of [`Frustum::intersects_obb`]
    /// where the transform is [`Affine3A::IDENTITY`] and both `intersect_near` and `intersect_far` are `true`.
    #[inline]
    pub fn intersects_obb_identity(&self, aabb: &Aabb) -> bool {
        let aabb_center_world = aabb.center.extend(1.0);
        for half_space in self.half_spaces.iter() {
            let p_normal = half_space.normal();
            let relative_radius = aabb.half_extents.abs().dot(p_normal.abs());
            if half_space.normal_d().dot(aabb_center_world) + relative_radius <= 0.0 {
                return false;
            }
        }
        true
    }

    /// Check if the frustum contains the entire Axis-Aligned Bounding Box (AABB).
    /// Referenced from: [Frustum Culling](https://learnopengl.com/Guest-Articles/2021/Scene/Frustum-Culling)
    #[inline]
    pub fn contains_aabb(&self, aabb: &Aabb, world_from_local: &Affine3A) -> bool {
        for half_space in &self.half_spaces {
            if !aabb.is_in_half_space(half_space, world_from_local) {
                return false;
            }
        }
        true
    }

    /// Optimized version of [`Self::contains_aabb`] when the AABB is already in world space.
    /// Use this when `world_from_local` would be [`Affine3A::IDENTITY`].
    #[inline]
    pub fn contains_aabb_identity(&self, aabb: &Aabb) -> bool {
        for half_space in &self.half_spaces {
            if !aabb.is_in_half_space_identity(half_space) {
                return false;
            }
        }
        true
    }
}

pub struct CubeMapFace {
    pub target: Vec3,
    pub up: Vec3,
}

// Cubemap faces are [+X, -X, +Y, -Y, +Z, -Z], per https://www.w3.org/TR/webgpu/#texture-view-creation
// Note: Cubemap coordinates are left-handed y-up, unlike the rest of Bevy.
// See https://registry.khronos.org/vulkan/specs/1.2/html/chap16.html#_cube_map_face_selection
//
// For each cubemap face, we take care to specify the appropriate target/up axis such that the rendered
// texture using Bevy's right-handed y-up coordinate space matches the expected cubemap face in
// left-handed y-up cubemap coordinates.
pub const CUBE_MAP_FACES: [CubeMapFace; 6] = [
    // +X
    CubeMapFace {
        target: Vec3::X,
        up: Vec3::Y,
    },
    // -X
    CubeMapFace {
        target: Vec3::NEG_X,
        up: Vec3::Y,
    },
    // +Y
    CubeMapFace {
        target: Vec3::Y,
        up: Vec3::Z,
    },
    // -Y
    CubeMapFace {
        target: Vec3::NEG_Y,
        up: Vec3::NEG_Z,
    },
    // +Z (with left-handed conventions, pointing forwards)
    CubeMapFace {
        target: Vec3::NEG_Z,
        up: Vec3::Y,
    },
    // -Z (with left-handed conventions, pointing backwards)
    CubeMapFace {
        target: Vec3::Z,
        up: Vec3::Y,
    },
];

pub fn face_index_to_name(face_index: usize) -> &'static str {
    match face_index {
        0 => "+x",
        1 => "-x",
        2 => "+y",
        3 => "-y",
        4 => "+z",
        5 => "-z",
        _ => "invalid",
    }
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default, Debug, Clone)]
pub struct CubemapFrusta {
    pub frusta: [Frustum; 6],
}

impl CubemapFrusta {
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Frustum> {
        self.frusta.iter()
    }
    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut Frustum> {
        self.frusta.iter_mut()
    }
}

/// Cubemap layout defines the order of images in a packed cubemap image.
#[derive(Default, Reflect, Debug, Clone, Copy)]
pub enum CubemapLayout {
    /// layout in a vertical cross format
    /// ```text
    ///    +y
    /// -x -z +x
    ///    -y
    ///    +z
    /// ```
    #[default]
    CrossVertical = 0,
    /// layout in a horizontal cross format
    /// ```text
    ///    +y
    /// -x -z +x +z
    ///    -y
    /// ```
    CrossHorizontal = 1,
    /// layout in a vertical sequence
    /// ```text
    ///   +x
    ///   -x
    ///   +y
    ///   -y
    ///   -z
    ///   +z
    /// ```
    SequenceVertical = 2,
    /// layout in a horizontal sequence
    /// ```text
    /// +x -x +y -y -z +z
    /// ```
    SequenceHorizontal = 3,
}

#[derive(Component, Debug, Default, Reflect, Clone)]
#[reflect(Component, Default, Debug, Clone)]
pub struct CascadesFrusta {
    pub frusta: EntityHashMap<Vec<Frustum>>,
}

#[cfg(test)]
mod tests {
    use core::f32::consts::PI;

    use bevy_math::{ops, EulerRot, Quat, Vec4};
    use bevy_transform::components::GlobalTransform;

    use crate::{CameraProjection, PerspectiveProjection};

    use super::*;

    // A big, offset frustum
    fn big_frustum() -> Frustum {
        Frustum(ViewFrustum {
            half_spaces: [
                HalfSpace::new(Vec4::new(-0.9701, -0.2425, -0.0000, 7.7611)),
                HalfSpace::new(Vec4::new(-0.0000, 1.0000, -0.0000, 4.0000)),
                HalfSpace::new(Vec4::new(-0.0000, -0.2425, -0.9701, 2.9104)),
                HalfSpace::new(Vec4::new(-0.0000, -1.0000, -0.0000, 4.0000)),
                HalfSpace::new(Vec4::new(-0.0000, -0.2425, 0.9701, 2.9104)),
                HalfSpace::new(Vec4::new(0.9701, -0.2425, -0.0000, -1.9403)),
            ],
        })
    }

    #[test]
    fn intersects_sphere_big_frustum_outside() {
        // Sphere outside frustum
        let frustum = big_frustum();
        let sphere = Sphere {
            center: Vec3A::new(0.9167, 0.0000, 0.0000),
            radius: 0.7500,
        };
        assert!(!frustum.intersects_sphere(&sphere, true));
    }

    #[test]
    fn intersects_sphere_big_frustum_intersect() {
        // Sphere intersects frustum boundary
        let frustum = big_frustum();
        let sphere = Sphere {
            center: Vec3A::new(7.9288, 0.0000, 2.9728),
            radius: 2.0000,
        };
        assert!(frustum.intersects_sphere(&sphere, true));
    }

    // A frustum
    fn frustum() -> Frustum {
        Frustum(ViewFrustum {
            half_spaces: [
                HalfSpace::new(Vec4::new(-0.9701, -0.2425, -0.0000, 0.7276)),
                HalfSpace::new(Vec4::new(-0.0000, 1.0000, -0.0000, 1.0000)),
                HalfSpace::new(Vec4::new(-0.0000, -0.2425, -0.9701, 0.7276)),
                HalfSpace::new(Vec4::new(-0.0000, -1.0000, -0.0000, 1.0000)),
                HalfSpace::new(Vec4::new(-0.0000, -0.2425, 0.9701, 0.7276)),
                HalfSpace::new(Vec4::new(0.9701, -0.2425, -0.0000, 0.7276)),
            ],
        })
    }

    #[test]
    fn intersects_sphere_frustum_surrounding() {
        // Sphere surrounds frustum
        let frustum = frustum();
        let sphere = Sphere {
            center: Vec3A::new(0.0000, 0.0000, 0.0000),
            radius: 3.0000,
        };
        assert!(frustum.intersects_sphere(&sphere, true));
    }

    #[test]
    fn intersects_sphere_frustum_contained() {
        // Sphere is contained in frustum
        let frustum = frustum();
        let sphere = Sphere {
            center: Vec3A::new(0.0000, 0.0000, 0.0000),
            radius: 0.7000,
        };
        assert!(frustum.intersects_sphere(&sphere, true));
    }

    #[test]
    fn intersects_sphere_frustum_intersects_plane() {
        // Sphere intersects a plane
        let frustum = frustum();
        let sphere = Sphere {
            center: Vec3A::new(0.0000, 0.0000, 0.9695),
            radius: 0.7000,
        };
        assert!(frustum.intersects_sphere(&sphere, true));
    }

    #[test]
    fn intersects_sphere_frustum_intersects_2_planes() {
        // Sphere intersects 2 planes
        let frustum = frustum();
        let sphere = Sphere {
            center: Vec3A::new(1.2037, 0.0000, 0.9695),
            radius: 0.7000,
        };
        assert!(frustum.intersects_sphere(&sphere, true));
    }

    #[test]
    fn intersects_sphere_frustum_intersects_3_planes() {
        // Sphere intersects 3 planes
        let frustum = frustum();
        let sphere = Sphere {
            center: Vec3A::new(1.2037, -1.0988, 0.9695),
            radius: 0.7000,
        };
        assert!(frustum.intersects_sphere(&sphere, true));
    }

    #[test]
    fn intersects_sphere_frustum_dodges_1_plane() {
        // Sphere avoids intersecting the frustum by 1 plane
        let frustum = frustum();
        let sphere = Sphere {
            center: Vec3A::new(-1.7020, 0.0000, 0.0000),
            radius: 0.7000,
        };
        assert!(!frustum.intersects_sphere(&sphere, true));
    }

    // A long frustum.
    fn long_frustum() -> Frustum {
        Frustum(ViewFrustum {
            half_spaces: [
                HalfSpace::new(Vec4::new(-0.9998, -0.0222, -0.0000, -1.9543)),
                HalfSpace::new(Vec4::new(-0.0000, 1.0000, -0.0000, 45.1249)),
                HalfSpace::new(Vec4::new(-0.0000, -0.0168, -0.9999, 2.2718)),
                HalfSpace::new(Vec4::new(-0.0000, -1.0000, -0.0000, 45.1249)),
                HalfSpace::new(Vec4::new(-0.0000, -0.0168, 0.9999, 2.2718)),
                HalfSpace::new(Vec4::new(0.9998, -0.0222, -0.0000, 7.9528)),
            ],
        })
    }

    #[test]
    fn intersects_sphere_long_frustum_outside() {
        // Sphere outside frustum
        let frustum = long_frustum();
        let sphere = Sphere {
            center: Vec3A::new(-4.4889, 46.9021, 0.0000),
            radius: 0.7500,
        };
        assert!(!frustum.intersects_sphere(&sphere, true));
    }

    #[test]
    fn intersects_sphere_long_frustum_intersect() {
        // Sphere intersects frustum boundary
        let frustum = long_frustum();
        let sphere = Sphere {
            center: Vec3A::new(-4.9957, 0.0000, -0.7396),
            radius: 4.4094,
        };
        assert!(frustum.intersects_sphere(&sphere, true));
    }

    #[test]
    fn sphere_intersects_obb_identical_center() {
        let sphere_at_origin = Sphere {
            center: Vec3A::ZERO,
            radius: 1.0,
        };
        let aabb_at_origin = Aabb {
            center: Vec3A::ZERO,
            half_extents: Vec3A::splat(0.5),
        };
        assert!(
            sphere_at_origin.intersects_obb(&aabb_at_origin, &Affine3A::IDENTITY),
            "Should intersect when centers are exactly identical"
        );
    }

    #[test]
    fn sphere_intersects_obb_at_edge() {
        // Zero-radius sphere (a point) exactly on the edge of an OBB
        let point_sphere = Sphere {
            center: Vec3A::new(1.0, 0.0, 0.0),
            radius: 0.0,
        };
        let aabb = Aabb {
            center: Vec3A::ZERO,
            half_extents: Vec3A::X, // Width of 1, height/depth 0
        };
        assert!(
            point_sphere.intersects_obb(&aabb, &Affine3A::IDENTITY),
            "Zero radius sphere (point) on the boundary should count as an intersection"
        );
    }

    #[test]
    fn sphere_intersects_obb_zero_extents_inside() {
        // OBB with zero extents (a point) inside a sphere
        let sphere = Sphere {
            center: Vec3A::ZERO,
            radius: 10.0,
        };
        let point_aabb = Aabb {
            center: Vec3A::new(1.0, 1.0, 1.0),
            half_extents: Vec3A::ZERO,
        };
        assert!(
            sphere.intersects_obb(&point_aabb, &Affine3A::IDENTITY),
            "Sphere should intersect an AABB with zero extents if the point is inside"
        );
    }

    #[test]
    fn sphere_intersects_obb_rotated_zeros() {
        // Rotated zero-extent OBB
        let sphere = Sphere {
            center: Vec3A::new(5.0, 0.0, 0.0),
            radius: 1.0,
        };
        let point_aabb = Aabb {
            center: Vec3A::ZERO,
            half_extents: Vec3A::ZERO,
        };

        // Rotate and translate the "point" OBB so it sits inside the sphere
        let transform = Affine3A::from_rotation_translation(
            Quat::from_rotation_y(PI),
            Vec3::new(5.0, 0.0, 0.0),
        );

        assert!(
            sphere.intersects_obb(&point_aabb, &transform),
            "Should intersect rotated point OBB"
        );
    }

    #[test]
    fn aabb_enclosing() {
        assert_eq!(Aabb::enclosing([] as [Vec3; 0]), None);
        assert_eq!(
            Aabb::enclosing(vec![Vec3::ONE]).unwrap(),
            Aabb::from_min_max(Vec3::ONE, Vec3::ONE)
        );
        assert_eq!(
            Aabb::enclosing(&[Vec3::Y, Vec3::X, Vec3::Z][..]).unwrap(),
            Aabb::from_min_max(Vec3::ZERO, Vec3::ONE)
        );
        assert_eq!(
            Aabb::enclosing([
                Vec3::NEG_X,
                Vec3::X * 2.0,
                Vec3::NEG_Y * 5.0,
                Vec3::Z,
                Vec3::ZERO
            ])
            .unwrap(),
            Aabb::from_min_max(Vec3::new(-1.0, -5.0, 0.0), Vec3::new(2.0, 0.0, 1.0))
        );
    }

    // A frustum with an offset for testing the [`Frustum::contains_aabb`] algorithm.
    fn contains_aabb_test_frustum() -> Frustum {
        let proj = PerspectiveProjection {
            fov: 90.0_f32.to_radians(),
            aspect_ratio: 1.0,
            near: 1.0,
            far: 100.0,
            ..PerspectiveProjection::default()
        };
        proj.compute_frustum(&GlobalTransform::from_translation(Vec3::new(2.0, 2.0, 0.0)))
    }

    fn contains_aabb_test_frustum_with_rotation() -> Frustum {
        let half_extent_world = (((49.5 * 49.5) * 0.5) as f32).sqrt() + 0.5f32.sqrt();
        let near = 50.5 - half_extent_world;
        let far = near + 2.0 * half_extent_world;
        let fov = 2.0 * ops::atan(half_extent_world / near);
        let proj = PerspectiveProjection {
            aspect_ratio: 1.0,
            near,
            far,
            fov,
            ..PerspectiveProjection::default()
        };
        proj.compute_frustum(&GlobalTransform::IDENTITY)
    }

    #[test]
    fn aabb_inside_frustum() {
        let frustum = contains_aabb_test_frustum();
        let aabb = Aabb {
            center: Vec3A::ZERO,
            half_extents: Vec3A::new(0.99, 0.99, 49.49),
        };
        let model = Affine3A::from_translation(Vec3::new(2.0, 2.0, -50.5));
        assert!(frustum.contains_aabb(&aabb, &model));
    }

    #[test]
    fn aabb_intersect_frustum() {
        let frustum = contains_aabb_test_frustum();
        let aabb = Aabb {
            center: Vec3A::ZERO,
            half_extents: Vec3A::new(0.99, 0.99, 49.6),
        };
        let model = Affine3A::from_translation(Vec3::new(2.0, 2.0, -50.5));
        assert!(!frustum.contains_aabb(&aabb, &model));
    }

    #[test]
    fn aabb_outside_frustum() {
        let frustum = contains_aabb_test_frustum();
        let aabb = Aabb {
            center: Vec3A::ZERO,
            half_extents: Vec3A::new(0.99, 0.99, 0.99),
        };
        let model = Affine3A::from_translation(Vec3::new(0.0, 0.0, 49.6));
        assert!(!frustum.contains_aabb(&aabb, &model));
    }

    #[test]
    fn aabb_inside_frustum_rotation() {
        let frustum = contains_aabb_test_frustum_with_rotation();
        let aabb = Aabb {
            center: Vec3A::new(0.0, 0.0, 0.0),
            half_extents: Vec3A::new(0.99, 0.99, 49.49),
        };

        let model = Affine3A::from_rotation_translation(
            Quat::from_rotation_x(PI / 4.0),
            Vec3::new(0.0, 0.0, -50.5),
        );
        assert!(frustum.contains_aabb(&aabb, &model));
    }

    #[test]
    fn aabb_intersect_frustum_rotation() {
        let frustum = contains_aabb_test_frustum_with_rotation();
        let aabb = Aabb {
            center: Vec3A::new(0.0, 0.0, 0.0),
            half_extents: Vec3A::new(0.99, 0.99, 49.6),
        };

        let model = Affine3A::from_rotation_translation(
            Quat::from_rotation_x(PI / 4.0),
            Vec3::new(0.0, 0.0, -50.5),
        );
        assert!(!frustum.contains_aabb(&aabb, &model));
    }

    #[test]
    fn test_identity_optimized_equivalence() {
        let cases = vec![
            (
                Aabb {
                    center: Vec3A::ZERO,
                    half_extents: Vec3A::splat(1.0),
                },
                HalfSpace::new(Vec4::new(1.0, 0.0, 0.0, -0.5)),
            ),
            (
                Aabb {
                    center: Vec3A::new(2.0, -1.0, 0.5),
                    half_extents: Vec3A::new(1.0, 2.0, 0.5),
                },
                HalfSpace::new(Vec4::new(1.0, 1.0, 1.0, -1.0).normalize()),
            ),
            (
                Aabb {
                    center: Vec3A::new(1.0, 1.0, 1.0),
                    half_extents: Vec3A::ZERO,
                },
                HalfSpace::new(Vec4::new(0.0, 0.0, 1.0, -2.0)),
            ),
        ];
        for (aabb, half_space) in cases {
            let general = aabb.is_in_half_space(&half_space, &Affine3A::IDENTITY);
            let identity = aabb.is_in_half_space_identity(&half_space);
            assert_eq!(general, identity,);
        }
    }

    #[test]
    fn intersects_obb_identity_matches_standard_true_true() {
        let frusta = [frustum(), long_frustum(), big_frustum()];
        let aabbs = [
            Aabb {
                center: Vec3A::ZERO,
                half_extents: Vec3A::new(0.5, 0.5, 0.5),
            },
            Aabb {
                center: Vec3A::new(1.0, 0.0, 0.5),
                half_extents: Vec3A::new(0.9, 0.9, 0.9),
            },
            Aabb {
                center: Vec3A::new(100.0, 100.0, 100.0),
                half_extents: Vec3A::new(1.0, 1.0, 1.0),
            },
        ];
        for fr in &frusta {
            for aabb in &aabbs {
                let standard = fr.intersects_obb(aabb, &Affine3A::IDENTITY, true, true);
                let optimized = fr.intersects_obb_identity(aabb);
                assert_eq!(standard, optimized);
            }
        }
    }

    /// Pseudo-random boxes around the test frusta, with rotations, non-uniform scales and shears.
    fn random_obbs() -> Vec<(Aabb, Affine3A)> {
        // A small xorshift generator, so that the tests are deterministic.
        let mut state = 0x2545_f491_u32;
        let mut random = move |min: f32, max: f32| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            min + (max - min) * (state as f32 / u32::MAX as f32)
        };
        (0..2000)
            .map(|_| {
                let aabb = Aabb {
                    center: Vec3A::new(random(-1.0, 1.0), random(-1.0, 1.0), random(-1.0, 1.0)),
                    half_extents: Vec3A::new(random(0.0, 2.0), random(0.0, 2.0), random(0.0, 2.0)),
                };
                let mut world_from_local = Affine3A::from_scale_rotation_translation(
                    Vec3::new(random(0.1, 3.0), random(0.1, 3.0), random(0.1, 3.0)),
                    Quat::from_euler(
                        EulerRot::YXZ,
                        random(-PI, PI),
                        random(-PI, PI),
                        random(-PI, PI),
                    ),
                    Vec3::new(
                        random(-15.0, 15.0),
                        random(-15.0, 15.0),
                        random(-15.0, 15.0),
                    ),
                );
                // Shear, as with an entity rotated under a parent with a non-uniform scale.
                world_from_local.matrix3.y_axis +=
                    world_from_local.matrix3.x_axis * random(-1.0, 1.0);
                (aabb, world_from_local)
            })
            .collect()
    }

    #[test]
    fn intersects_obb_with_bounding_sphere_matches_sphere_then_obb() {
        for frustum in [frustum(), long_frustum(), big_frustum()] {
            for (aabb, world_from_local) in random_obbs() {
                let radius =
                    GlobalTransform::from(world_from_local).radius_vec3a(aabb.half_extents);
                let sphere = Sphere {
                    center: world_from_local.transform_point3a(aabb.center),
                    radius,
                };
                for intersect_far in [false, true] {
                    assert_eq!(
                        frustum.intersects_obb_with_bounding_sphere(
                            &aabb,
                            &world_from_local,
                            radius,
                            true,
                            intersect_far
                        ),
                        frustum.intersects_sphere(&sphere, intersect_far)
                            && frustum.intersects_obb(
                                &aabb,
                                &world_from_local,
                                true,
                                intersect_far
                            ),
                    );
                }
            }
        }
    }

    #[test]
    fn intersects_obb_with_bounding_sphere_matches_obb_when_the_sphere_contains_it() {
        for frustum in [frustum(), long_frustum(), big_frustum()] {
            for (aabb, world_from_local) in random_obbs() {
                let radius = aabb.bounding_sphere_radius(&world_from_local.matrix3);
                for (intersect_near, intersect_far) in
                    [(false, false), (false, true), (true, false), (true, true)]
                {
                    assert_eq!(
                        frustum.intersects_obb_with_bounding_sphere(
                            &aabb,
                            &world_from_local,
                            radius,
                            intersect_near,
                            intersect_far
                        ),
                        frustum.intersects_obb(
                            &aabb,
                            &world_from_local,
                            intersect_near,
                            intersect_far
                        ),
                    );
                }
            }
        }
    }

    #[test]
    fn bounding_sphere_radius_contains_the_corners() {
        for (aabb, world_from_local) in random_obbs() {
            let center = world_from_local.transform_point3a(aabb.center);
            let radius = aabb.bounding_sphere_radius(&world_from_local.matrix3);
            for x in [-1.0, 1.0] {
                for y in [-1.0, 1.0] {
                    for z in [-1.0, 1.0] {
                        let corner = world_from_local.transform_point3a(
                            aabb.center + aabb.half_extents * Vec3A::new(x, y, z),
                        );
                        assert!(corner.distance(center) <= radius * (1.0 + 1e-5));
                    }
                }
            }
        }
    }
}
