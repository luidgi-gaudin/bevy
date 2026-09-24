//! A bounding volume hierarchy over the triangles of a mesh, used to speed up ray casts.

use alloc::vec::Vec;
use bevy_math::{Vec3, Vec3A};
use bevy_shape::Ray3d;

use super::intersections::{ray_triangle_intersection, RayTriangleHit};
use super::Backfaces;

/// Leaves with at most this many triangles are never split.
const MIN_SPLIT_TRIANGLES: usize = 4;

/// Leaves with more than this many triangles are always split, if possible.
const MAX_LEAF_TRIANGLES: usize = 16;

/// Number of bins used to evaluate the surface area heuristic along each axis.
const SAH_BINS: usize = 12;

/// Cost of traversing a node, relative to the cost of intersecting a triangle.
const TRAVERSAL_COST: f32 = 1.0;

/// A node of a [`TriangleBvh`].
#[derive(Clone, Copy, Debug)]
struct Node {
    /// The minimum corner of the bounds of the node's triangles.
    min: Vec3,
    /// For a leaf, the index of its first triangle in [`TriangleBvh::triangles`].
    /// For an inner node, the index of its first child: the second child is right after it.
    first: u32,
    /// The maximum corner of the bounds of the node's triangles.
    max: Vec3,
    /// The number of triangles of a leaf, or 0 for an inner node.
    triangle_count: u32,
}

/// A bounding volume hierarchy (BVH) over the triangles of a mesh.
///
/// Instead of testing a ray against all the triangles of a mesh, a BVH makes it possible to only
/// test the triangles in the nodes of the tree that the ray goes through, closest first. This
/// makes ray casts against large meshes orders of magnitude faster.
///
/// The nodes are split with the surface area heuristic (SAH).
#[derive(Debug, Default)]
pub struct TriangleBvh {
    nodes: Vec<Node>,
    /// Triangle indices, ordered so that the triangles of each leaf are contiguous.
    triangles: Vec<u32>,
    /// Triangles with non-finite vertex positions, which can't be placed in the tree. They are
    /// tested for each ray.
    unbounded_triangles: Vec<u32>,
    /// The number of triangles the BVH was built for.
    triangle_count: usize,
}

/// A triangle being placed in a [`TriangleBvh`]. This is kept small, as triangles are moved
/// around a lot while building the tree.
#[derive(Clone, Copy)]
struct BuildTriangle {
    min: Vec3,
    index: u32,
    max: Vec3,
}

impl BuildTriangle {
    /// The center of the bounds of the triangle, used to sort triangles in the tree.
    #[inline]
    fn center(&self) -> Vec3A {
        (Vec3A::from(self.min) + Vec3A::from(self.max)) * 0.5
    }
}

/// An axis-aligned bounding box that can be empty.
#[derive(Clone, Copy)]
struct Bounds {
    min: Vec3A,
    max: Vec3A,
}

impl Bounds {
    const EMPTY: Self = Self {
        min: Vec3A::INFINITY,
        max: Vec3A::NEG_INFINITY,
    };

    #[inline]
    fn grow(&mut self, min: Vec3A, max: Vec3A) {
        self.min = self.min.min(min);
        self.max = self.max.max(max);
    }

    #[inline]
    fn grow_bounds(&mut self, other: &Bounds) {
        self.grow(other.min, other.max);
    }

    /// Half of the surface area of the box, or 0 if it is empty.
    fn half_area(&self) -> f32 {
        let extent = (self.max - self.min).max(Vec3A::ZERO);
        extent.x * extent.y + extent.y * extent.z + extent.z * extent.x
    }
}

/// A bin used to evaluate the surface area heuristic.
#[derive(Clone, Copy)]
struct Bin {
    bounds: Bounds,
    count: usize,
}

impl Bin {
    const EMPTY: Self = Self {
        bounds: Bounds::EMPTY,
        count: 0,
    };
}

/// The bounds of the triangles of a node, and of their centers.
#[derive(Clone, Copy)]
struct NodeBounds {
    triangles: Bounds,
    centers: Bounds,
}

impl NodeBounds {
    fn of(triangles: &[BuildTriangle]) -> Self {
        let mut bounds = NodeBounds {
            triangles: Bounds::EMPTY,
            centers: Bounds::EMPTY,
        };
        for triangle in triangles {
            bounds.grow(triangle);
        }
        bounds
    }

    #[inline]
    fn grow(&mut self, triangle: &BuildTriangle) {
        self.triangles
            .grow(triangle.min.into(), triangle.max.into());
        let center = triangle.center();
        self.centers.grow(center, center);
    }
}

/// How to split the triangles of a node in two children.
struct Split {
    /// The number of triangles of the first child, which come first in the node's range.
    first_count: usize,
    first_bounds: NodeBounds,
    second_bounds: NodeBounds,
}

impl TriangleBvh {
    /// Builds a BVH over `triangle_count` triangles, whose vertex positions are given by
    /// `triangle`. Triangles for which `triangle` returns `None` are left out.
    pub fn new(triangle_count: usize, triangle: impl Fn(usize) -> Option<[Vec3; 3]>) -> Self {
        let mut triangles = Vec::with_capacity(triangle_count);
        let mut unbounded_triangles = Vec::new();
        for index in 0..triangle_count {
            let Some([a, b, c]) = triangle(index) else {
                continue;
            };
            let [a, b, c] = [Vec3A::from(a), Vec3A::from(b), Vec3A::from(c)];
            if !(a.is_finite() && b.is_finite() && c.is_finite()) {
                unbounded_triangles.push(index as u32);
                continue;
            }
            triangles.push(BuildTriangle {
                min: a.min(b).min(c).into(),
                index: index as u32,
                max: a.max(b).max(c).into(),
            });
        }

        let mut bvh = TriangleBvh {
            nodes: Vec::with_capacity(2 * triangles.len().div_ceil(MIN_SPLIT_TRIANGLES)),
            triangles: Vec::new(),
            unbounded_triangles,
            triangle_count,
        };
        if !triangles.is_empty() {
            bvh.build(&mut triangles);
            bvh.triangles = triangles.iter().map(|triangle| triangle.index).collect();
        }
        bvh
    }

    /// The number of triangles this BVH was built for.
    pub fn triangle_count(&self) -> usize {
        self.triangle_count
    }

    /// Builds the nodes of the tree, and reorders the triangles so that the triangles of each
    /// leaf are contiguous.
    fn build(&mut self, triangles: &mut [BuildTriangle]) {
        let bounds = NodeBounds::of(triangles);
        self.nodes
            .push(Self::node(&bounds.triangles, 0, triangles.len()));
        let mut stack = vec![(0, 0, triangles.len(), bounds)];

        while let Some((node_index, start, end, bounds)) = stack.pop() {
            if end - start <= MIN_SPLIT_TRIANGLES {
                continue;
            }
            let Some(split) = Self::split(&mut triangles[start..end], &bounds) else {
                continue;
            };
            let middle = start + split.first_count;

            let first_child = self.nodes.len();
            self.nodes
                .push(Self::node(&split.first_bounds.triangles, start, middle));
            self.nodes
                .push(Self::node(&split.second_bounds.triangles, middle, end));
            let node = &mut self.nodes[node_index];
            node.first = first_child as u32;
            node.triangle_count = 0;

            stack.push((first_child, start, middle, split.first_bounds));
            stack.push((first_child + 1, middle, end, split.second_bounds));
        }
    }

    /// Creates a leaf for the triangles in `start..end`, with the given bounds.
    fn node(bounds: &Bounds, start: usize, end: usize) -> Node {
        // Slightly enlarge the bounds, so that rounding errors in the ray-box test never make a
        // ray miss a box when it hits one of its triangles.
        let padding = (bounds.max - bounds.min).max_element() * 1e-5
            + bounds.min.abs().max(bounds.max.abs()).max_element() * 1e-6
            + f32::MIN_POSITIVE;
        Node {
            min: (bounds.min - padding).into(),
            first: start as u32,
            max: (bounds.max + padding).into(),
            triangle_count: (end - start) as u32,
        }
    }

    /// Splits the triangles in two groups using the surface area heuristic, and moves the
    /// triangles of the first group first. Returns `None` if the triangles should stay in a
    /// single leaf.
    fn split(triangles: &mut [BuildTriangle], bounds: &NodeBounds) -> Option<Split> {
        let center_bounds = &bounds.centers;
        let extent = center_bounds.max - center_bounds.min;
        // Only look for a split plane along the axis in which the triangles are the most spread out.
        // Binning along the other axes too gives slightly better trees, but is much slower to build.
        let axis = extent.max_position();
        if extent[axis] <= 0.0 {
            return (triangles.len() > MAX_LEAF_TRIANGLES).then(|| {
                // All the triangles have the same center, but there are too many triangles for a
                // leaf: split them in two halves.
                let first_count = triangles.len() / 2;
                Split {
                    first_count,
                    first_bounds: NodeBounds::of(&triangles[..first_count]),
                    second_bounds: NodeBounds::of(&triangles[first_count..]),
                }
            });
        }
        let scale = SAH_BINS as f32 / extent[axis];
        let min = center_bounds.min[axis];
        let bin_of = |triangle: &BuildTriangle| {
            // Truncating is the intent here, and negative values saturate to 0.
            let center = (triangle.min[axis] + triangle.max[axis]) * 0.5;
            (((center - min) * scale) as usize).min(SAH_BINS - 1)
        };

        let mut bins = [Bin::EMPTY; SAH_BINS];
        for triangle in triangles.iter() {
            let bin = &mut bins[bin_of(triangle)];
            bin.bounds.grow(triangle.min.into(), triangle.max.into());
            bin.count += 1;
        }

        // Sweep from the right to get the bounds of the second group of each split plane.
        let mut second_group_bounds = [Bounds::EMPTY; SAH_BINS];
        let mut sweep_bounds = Bounds::EMPTY;
        for split in (1..SAH_BINS).rev() {
            sweep_bounds.grow_bounds(&bins[split].bounds);
            second_group_bounds[split] = sweep_bounds;
        }

        // Find the split plane with the lowest cost.
        let mut best: Option<(usize, f32)> = None;
        let mut first_group_bounds = Bounds::EMPTY;
        let mut first_count = 0;
        for split in 1..SAH_BINS {
            first_group_bounds.grow_bounds(&bins[split - 1].bounds);
            first_count += bins[split - 1].count;
            if first_count == 0 || first_count == triangles.len() {
                continue;
            }
            let cost = first_group_bounds.half_area() * first_count as f32
                + second_group_bounds[split].half_area() * (triangles.len() - first_count) as f32;
            if best.is_none_or(|(_, best_cost)| cost < best_cost) {
                best = Some((split, cost));
            }
        }

        let area = bounds.triangles.half_area();
        let (split, cost) = best?;
        let split_cost = TRAVERSAL_COST + if area > 0.0 { cost / area } else { 0.0 };
        if triangles.len() <= MAX_LEAF_TRIANGLES && split_cost >= triangles.len() as f32 {
            return None;
        }

        // Move the triangles of the first group first, and compute the bounds of both groups.
        let mut first_bounds = NodeBounds {
            triangles: Bounds::EMPTY,
            centers: Bounds::EMPTY,
        };
        let mut second_bounds = first_bounds;
        let mut first_count = 0;
        for i in 0..triangles.len() {
            let triangle = triangles[i];
            if bin_of(&triangle) < split {
                first_bounds.grow(&triangle);
                triangles.swap(first_count, i);
                first_count += 1;
            } else {
                second_bounds.grow(&triangle);
            }
        }
        Some(Split {
            first_count,
            first_bounds,
            second_bounds,
        })
    }

    /// Returns the index of the closest triangle hit by the ray, along with the hit.
    ///
    /// This returns the exact same result as testing all the triangles in order and keeping the
    /// first one with the smallest non-negative distance: when several triangles are hit at the
    /// same distance, the one with the smallest index is returned.
    pub(super) fn closest_hit(
        &self,
        ray: &Ray3d,
        backfaces: Backfaces,
        triangle: impl Fn(usize) -> Option<[Vec3; 3]>,
    ) -> Option<(usize, RayTriangleHit)> {
        let mut closest: Option<(usize, RayTriangleHit)> = None;
        let mut closest_distance = f32::MAX;
        let mut test_triangle = |index: u32, closest_distance: &mut f32| {
            let index = index as usize;
            let Some(vertices) = triangle(index) else {
                return;
            };
            let Some(hit) = ray_triangle_intersection(ray, &vertices, backfaces) else {
                return;
            };
            if hit.distance >= 0.0
                && (hit.distance < *closest_distance
                    || (hit.distance == *closest_distance
                        && closest
                            .as_ref()
                            .is_some_and(|(closest, _)| index < *closest)))
            {
                *closest_distance = hit.distance;
                closest = Some((index, hit));
            }
        };

        for &index in &self.unbounded_triangles {
            test_triangle(index, &mut closest_distance);
        }

        if self.nodes.is_empty() {
            return closest;
        }

        let origin = Vec3A::from(ray.origin);
        let direction = Vec3A::from(*ray.direction);
        let inverse_direction = direction.recip();
        let ray_box = |node: &Node| {
            ray_box_intersection(origin, direction, inverse_direction, node.min, node.max)
        };

        let mut stack: Vec<(u32, f32)> = Vec::with_capacity(64);
        if let Some(distance) = ray_box(&self.nodes[0]) {
            stack.push((0, distance));
        }
        while let Some((node_index, distance)) = stack.pop() {
            // Triangles at the same distance as the closest hit could have a smaller index.
            if distance > closest_distance {
                continue;
            }
            let node = &self.nodes[node_index as usize];
            if node.triangle_count > 0 {
                let start = node.first as usize;
                for &index in &self.triangles[start..start + node.triangle_count as usize] {
                    test_triangle(index, &mut closest_distance);
                }
                continue;
            }

            // Visit the closest child first, by pushing it last.
            let first = node.first;
            let second = node.first + 1;
            match (
                ray_box(&self.nodes[first as usize]),
                ray_box(&self.nodes[second as usize]),
            ) {
                (Some(first_distance), Some(second_distance)) => {
                    if first_distance <= second_distance {
                        stack.push((second, second_distance));
                        stack.push((first, first_distance));
                    } else {
                        stack.push((first, first_distance));
                        stack.push((second, second_distance));
                    }
                }
                (Some(first_distance), None) => stack.push((first, first_distance)),
                (None, Some(second_distance)) => stack.push((second, second_distance)),
                (None, None) => {}
            }
        }

        closest
    }
}

/// Returns the distance along the ray at which it enters the box, or 0 if it starts inside the box,
/// or `None` if the ray misses the box.
#[inline]
fn ray_box_intersection(
    origin: Vec3A,
    direction: Vec3A,
    inverse_direction: Vec3A,
    min: Vec3,
    max: Vec3,
) -> Option<f32> {
    let min = Vec3A::from(min);
    let max = Vec3A::from(max);
    let t0 = (min - origin) * inverse_direction;
    let t1 = (max - origin) * inverse_direction;
    let near = t0.min(t1);
    let far = t0.max(t1);

    let mut t_near = 0.0f32;
    let mut t_far = f32::INFINITY;
    for axis in 0..3 {
        if direction[axis] == 0.0 {
            // The ray is parallel to the slab: it must start between its planes.
            if origin[axis] < min[axis] || origin[axis] > max[axis] {
                return None;
            }
        } else {
            t_near = t_near.max(near[axis]);
            t_far = t_far.min(far[axis]);
        }
    }
    (t_near <= t_far).then_some(t_near)
}

#[cfg(test)]
mod tests {
    use super::TriangleBvh;
    use crate::mesh_picking::ray_cast::{intersections::ray_triangle_intersection, Backfaces};
    use alloc::vec::Vec;
    use bevy_math::{Dir3, Vec3};
    use bevy_shape::Ray3d;

    /// A small deterministic random number generator.
    struct Rng(u64);

    impl Rng {
        fn next_f32(&mut self) -> f32 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (self.0 >> 40) as f32 / (1u64 << 24) as f32
        }

        fn vec3(&mut self, scale: f32) -> Vec3 {
            (Vec3::new(self.next_f32(), self.next_f32(), self.next_f32()) * 2.0 - 1.0) * scale
        }
    }

    /// The closest hit, found by testing all the triangles in order.
    fn linear_closest_hit(
        triangles: &[[Vec3; 3]],
        ray: &Ray3d,
        backfaces: Backfaces,
    ) -> Option<(usize, f32)> {
        let mut closest = None;
        let mut closest_distance = f32::MAX;
        for (index, triangle) in triangles.iter().enumerate() {
            if let Some(hit) = ray_triangle_intersection(ray, triangle, backfaces)
                && hit.distance >= 0.0
                && hit.distance < closest_distance
            {
                closest_distance = hit.distance;
                closest = Some((index, hit.distance));
            }
        }
        closest
    }

    fn assert_same_hits(triangles: &[[Vec3; 3]], rays: &[Ray3d]) {
        let bvh = TriangleBvh::new(triangles.len(), |i| triangles.get(i).copied());
        for ray in rays {
            for backfaces in [Backfaces::Cull, Backfaces::Include] {
                let expected = linear_closest_hit(triangles, ray, backfaces);
                let actual = bvh
                    .closest_hit(ray, backfaces, |i| triangles.get(i).copied())
                    .map(|(index, hit)| (index, hit.distance));
                assert_eq!(expected, actual, "{ray:?}");
            }
        }
    }

    #[test]
    fn random_triangles() {
        let mut rng = Rng(1);
        let triangles: Vec<[Vec3; 3]> = (0..2000)
            .map(|_| {
                let center = rng.vec3(10.0);
                [
                    center + rng.vec3(0.5),
                    center + rng.vec3(0.5),
                    center + rng.vec3(0.5),
                ]
            })
            .collect();
        let rays: Vec<Ray3d> = (0..500)
            .map(|_| {
                let origin = rng.vec3(15.0);
                let target = rng.vec3(5.0);
                Ray3d::new(origin, Dir3::new(target - origin).unwrap())
            })
            .collect();
        assert_same_hits(&triangles, &rays);
    }

    #[test]
    fn grid_with_ties() {
        // A flat grid, where rays through vertices and edges hit several triangles at the same
        // distance, and axis-aligned rays have zero direction components.
        let size = 32;
        let mut triangles = Vec::new();
        for z in 0..size {
            for x in 0..size {
                let p = |dx: i32, dz: i32| Vec3::new((x + dx) as f32, 0.0, (z + dz) as f32);
                triangles.push([p(0, 0), p(0, 1), p(1, 0)]);
                triangles.push([p(1, 0), p(0, 1), p(1, 1)]);
            }
        }
        let mut rays = Vec::new();
        for z in 0..=2 * size {
            for x in 0..=2 * size {
                let origin = Vec3::new(x as f32 * 0.5, 1.0, z as f32 * 0.5);
                rays.push(Ray3d::new(origin, Dir3::NEG_Y));
                rays.push(Ray3d::new(
                    origin,
                    Dir3::new(Vec3::new(0.3, -1.0, 0.2)).unwrap(),
                ));
            }
        }
        // Rays in the plane of the grid.
        rays.push(Ray3d::new(Vec3::new(-1.0, 0.0, 3.0), Dir3::X));
        rays.push(Ray3d::new(Vec3::new(3.0, 0.0, -1.0), Dir3::Z));
        assert_same_hits(&triangles, &rays);
    }

    #[test]
    fn degenerate_and_non_finite_triangles() {
        let mut rng = Rng(2);
        let mut triangles: Vec<[Vec3; 3]> = (0..300)
            .map(|_| {
                let center = rng.vec3(3.0);
                [center, center + rng.vec3(1.0), center + rng.vec3(1.0)]
            })
            .collect();
        // Many copies of the same triangle, which can't be split by the SAH.
        triangles.extend(core::iter::repeat_n([Vec3::ZERO, Vec3::X, Vec3::Y], 100));
        // Points and lines.
        triangles.push([Vec3::ONE; 3]);
        triangles.push([Vec3::ZERO, Vec3::ONE, Vec3::ONE * 2.0]);
        // Non-finite positions.
        triangles.push([Vec3::NAN, Vec3::X, Vec3::Y]);
        triangles.push([Vec3::INFINITY, Vec3::X, Vec3::Y]);

        let rays: Vec<Ray3d> = (0..300)
            .map(|_| {
                let origin = rng.vec3(6.0);
                let target = rng.vec3(1.0);
                Ray3d::new(origin, Dir3::new(target - origin).unwrap())
            })
            .chain([Ray3d::new(Vec3::new(0.2, 0.2, 5.0), Dir3::NEG_Z)])
            .collect();
        assert_same_hits(&triangles, &rays);
    }

    #[test]
    fn missing_triangles_are_skipped() {
        let triangles = [
            [Vec3::ZERO, Vec3::X, Vec3::Y],
            [Vec3::Z, Vec3::Z + Vec3::X, Vec3::Z + Vec3::Y],
        ];
        // The first triangle can't be read, e.g. because of invalid indices.
        let triangle = |i: usize| (i == 1).then(|| triangles[i]);
        let bvh = TriangleBvh::new(2, triangle);
        let ray = Ray3d::new(Vec3::new(0.1, 0.1, 5.0), Dir3::NEG_Z);
        let hit = bvh.closest_hit(&ray, Backfaces::Include, triangle);
        assert_eq!(hit.map(|(index, _)| index), Some(1));
        assert!(TriangleBvh::new(0, triangle)
            .closest_hit(&ray, Backfaces::Include, triangle)
            .is_none());
    }
}
