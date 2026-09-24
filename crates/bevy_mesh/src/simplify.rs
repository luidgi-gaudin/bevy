//! Simplification of meshes, to generate levels of detail.
//!
//! [`Mesh::simplify`] reduces the number of triangles of a mesh by repeatedly collapsing edges,
//! merging one of their vertices into the other one. The cost of each collapse is measured with
//! quadric error metrics, as described in
//! [Surface Simplification Using Quadric Error Metrics](https://www.cs.cmu.edu/~garland/Papers/quadrics.pdf),
//! and the collapses that change the shape of the mesh the least are done first.
//!
//! Vertices are only ever merged into other existing vertices, so all their attributes (normals,
//! UVs, joint weights...) stay valid. Attribute seams (vertices that are split because they have
//! different attributes on each side, like the edges of UV islands) and borders are preserved,
//! collapses that would flip triangles are rejected, and the simplification stops before the shape
//! of the mesh changes more than a given error.

use crate::{
    optimize::validate_indices, Indices, Mesh, MeshOptimizationError, PrimitiveTopology,
    VertexAttributeValues,
};
use alloc::vec::Vec;
use bevy_math::DVec3;
use bevy_platform::collections::{HashMap, HashSet};
use core::ops::AddAssign;

/// Weight of the quadrics that keep borders and attribute seams in place, relative to the weight
/// of the quadrics of the triangles.
const EDGE_WEIGHT: f64 = 2.0;

/// Collapses that rotate the normal of a triangle by more than this angle (as its cosine) are
/// rejected, as they create visible artifacts and would flip triangles.
const MIN_NORMAL_COSINE: f64 = 0.25;

/// Settings for [`Mesh::simplify`].
#[derive(Clone, Debug, PartialEq)]
pub struct MeshSimplificationSettings {
    /// The fraction of the triangles of the mesh to keep, between 0 and 1.
    ///
    /// This is a target: fewer triangles are removed if removing more would move the surface of
    /// the mesh by more than [`max_error`](Self::max_error).
    pub target_ratio: f32,
    /// How much the surface of the mesh is allowed to move, relative to the size of the mesh
    /// (the largest dimension of its bounding box).
    ///
    /// For example, with `0.01`, the surface can move by up to 1% of the size of the mesh. Use
    /// `f32::INFINITY` to always reach [`target_ratio`](Self::target_ratio).
    pub max_error: f32,
    /// Prevents the vertices on the borders of the mesh (edges that are only used by one
    /// triangle) from moving.
    ///
    /// Use this for meshes that are parts of a larger surface, like terrain chunks, so that no
    /// gaps appear between them.
    pub lock_border: bool,
}

impl Default for MeshSimplificationSettings {
    fn default() -> Self {
        Self {
            target_ratio: 0.5,
            max_error: 0.01,
            lock_border: false,
        }
    }
}

impl Mesh {
    /// Reduces the number of triangles of this mesh, while keeping its shape as close as possible
    /// to the original one. This can be used to generate simpler meshes for objects that are far
    /// away from the camera, see [`VisibilityRange`](https://docs.rs/bevy/latest/bevy/camera/visibility/struct.VisibilityRange.html).
    ///
    /// Returns the error of the simplified mesh: how much its surface moved, relative to the size
    /// of the mesh, like [`MeshSimplificationSettings::max_error`].
    ///
    /// The remaining vertices keep all their attributes. The simplified mesh is optimized with
    /// [`Mesh::optimize_for_gpu`], which removes the vertices that are no longer used.
    ///
    /// Returns an error if the mesh isn't an indexed [`PrimitiveTopology::TriangleList`], if its
    /// positions aren't in the `Float32x3` format, if its indices are invalid, or if the mesh data
    /// has been extracted to the `RenderWorld`.
    ///
    /// ```
    /// # use bevy_mesh::{Mesh, Meshable, MeshSimplificationSettings};
    /// # use bevy_shape::Sphere;
    /// let mut mesh = Sphere::new(1.0).mesh().uv(64, 32);
    /// let triangles = mesh.indices().unwrap().len() / 3;
    ///
    /// let error = mesh
    ///     .simplify(&MeshSimplificationSettings {
    ///         target_ratio: 0.25,
    ///         ..Default::default()
    ///     })
    ///     .unwrap();
    ///
    /// assert!(mesh.indices().unwrap().len() / 3 <= triangles / 4);
    /// assert!(error <= 0.01);
    /// ```
    pub fn simplify(
        &mut self,
        settings: &MeshSimplificationSettings,
    ) -> Result<f32, MeshOptimizationError> {
        if self.primitive_topology() != PrimitiveTopology::TriangleList {
            return Err(MeshOptimizationError::UnsupportedTopology(
                self.primitive_topology(),
            ));
        }
        if settings.target_ratio.is_nan() || settings.max_error.is_nan() || settings.max_error < 0.0
        {
            return Err(MeshOptimizationError::InvalidSimplificationSettings(
                settings.clone(),
            ));
        }
        let vertex_count = self.try_count_vertices()?;
        self.validate_morph_targets(vertex_count)?;
        let Some(VertexAttributeValues::Float32x3(positions)) =
            self.try_attribute_option(Mesh::ATTRIBUTE_POSITION)?
        else {
            return Err(MeshOptimizationError::UnsupportedPositions);
        };
        let indices = self
            .try_indices_option()?
            .ok_or(MeshOptimizationError::MissingIndices)?;
        validate_indices(indices, vertex_count, PrimitiveTopology::TriangleList)?;

        let triangle_count = indices.len() / 3;
        let target_triangle_count =
            (triangle_count as f64 * f64::from(settings.target_ratio.clamp(0.0, 1.0))) as usize;
        let mut simplified_indices: Vec<u32> = indices.iter().map(|index| index as u32).collect();
        let error = simplify(
            &mut simplified_indices,
            positions,
            target_triangle_count,
            settings.max_error,
            settings.lock_border,
        );

        let simplified_indices = match indices {
            Indices::U16(_) => Indices::U16(
                simplified_indices
                    .iter()
                    .map(|&index| index as u16)
                    .collect(),
            ),
            Indices::U32(_) => Indices::U32(simplified_indices),
        };
        self.try_insert_indices(simplified_indices)?;
        self.optimize_for_gpu()?;
        Ok(error)
    }

    /// Consumes the mesh and returns a mesh simplified with [`Mesh::simplify`].
    ///
    /// Returns an error if the mesh isn't an indexed [`PrimitiveTopology::TriangleList`], if its
    /// positions aren't in the `Float32x3` format, if its indices are invalid, or if the mesh data
    /// has been extracted to the `RenderWorld`.
    pub fn simplified(
        mut self,
        settings: &MeshSimplificationSettings,
    ) -> Result<Self, MeshOptimizationError> {
        self.simplify(settings)?;
        Ok(self)
    }
}

/// A quadric, which measures the squared distance of a point to a set of weighted planes.
#[derive(Clone, Copy, Default)]
struct Quadric {
    xx: f64,
    yy: f64,
    zz: f64,
    xy: f64,
    xz: f64,
    yz: f64,
    x: f64,
    y: f64,
    z: f64,
    c: f64,
    weight: f64,
}

impl Quadric {
    /// The quadric of the plane with the given unit `normal` that contains `point`.
    fn from_plane(normal: DVec3, point: DVec3, weight: f64) -> Self {
        let d = -normal.dot(point);
        let n = normal * weight;
        Self {
            xx: n.x * normal.x,
            yy: n.y * normal.y,
            zz: n.z * normal.z,
            xy: n.x * normal.y,
            xz: n.x * normal.z,
            yz: n.y * normal.z,
            x: n.x * d,
            y: n.y * d,
            z: n.z * d,
            c: weight * d * d,
            weight,
        }
    }

    /// The weighted average of the squared distances of `point` to the planes of the quadric.
    fn error(&self, point: DVec3) -> f64 {
        if self.weight <= 0.0 {
            return 0.0;
        }
        let DVec3 { x, y, z } = point;
        let error = self.xx * x * x
            + self.yy * y * y
            + self.zz * z * z
            + 2.0 * (self.xy * x * y + self.xz * x * z + self.yz * y * z)
            + 2.0 * (self.x * x + self.y * y + self.z * z)
            + self.c;
        // The error can be slightly negative because of rounding errors.
        error.abs() / self.weight
    }
}

impl AddAssign for Quadric {
    fn add_assign(&mut self, other: Self) {
        self.xx += other.xx;
        self.yy += other.yy;
        self.zz += other.zz;
        self.xy += other.xy;
        self.xz += other.xz;
        self.yz += other.yz;
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
        self.c += other.c;
        self.weight += other.weight;
    }
}

/// How a position can be collapsed.
#[derive(Clone, Copy, PartialEq, Eq)]
enum PositionKind {
    /// The position is in the middle of the surface, and can be collapsed onto any neighbor.
    Manifold,
    /// The position is on a border of the surface, and can only be collapsed along the border.
    Border,
    /// The position can't be collapsed.
    Locked,
}

/// Simplifies a triangle list in place, merging vertices into other vertices, until it has at
/// most `target_triangle_count` triangles or until simplifying more would cause an error larger
/// than `max_error`.
///
/// Returns the error of the simplified mesh, relative to its size.
fn simplify(
    indices: &mut Vec<u32>,
    positions: &[[f32; 3]],
    target_triangle_count: usize,
    max_error: f32,
    lock_border: bool,
) -> f32 {
    // Work in a normalized space, so that errors are relative to the size of the mesh.
    let (min, max) = positions
        .iter()
        .map(|&position| DVec3::from(position.map(f64::from)))
        .filter(|position| position.is_finite())
        .fold((DVec3::INFINITY, DVec3::NEG_INFINITY), |(min, max), p| {
            (min.min(p), max.max(p))
        });
    let origin = if min.is_finite() { min } else { DVec3::ZERO };
    let extent = (max - min).max_element();
    let scale = if extent > 0.0 && extent.is_finite() {
        1.0 / extent
    } else {
        1.0
    };

    // Vertices with the same position are split because their other attributes are different.
    // Collapses are decided per position, and the vertices of a position are collapsed together.
    let mut position_ids = HashMap::<[u32; 3], u32>::default();
    let mut points = Vec::new();
    let position_of: Vec<u32> = positions
        .iter()
        .map(|position| {
            // Adding 0 turns -0 into +0, so that they compare equal.
            let key = position.map(|coordinate| (coordinate + 0.0).to_bits());
            *position_ids.entry(key).or_insert_with(|| {
                points.push((DVec3::from(position.map(f64::from)) - origin) * scale);
                points.len() as u32 - 1
            })
        })
        .collect();
    drop(position_ids);
    let position_count = points.len();

    // Remove the triangles that are already degenerate.
    let mut triangles: Vec<[u32; 3]> = indices
        .as_chunks::<3>()
        .0
        .iter()
        .copied()
        .filter(|triangle| {
            let [a, b, c] = triangle.map(|vertex| position_of[vertex as usize]);
            a != b && b != c && c != a
        })
        .collect();

    // Compute the quadric of each position, from the planes of its triangles, and from planes
    // perpendicular to its triangles along borders and attribute seams, to keep them in place.
    let mut quadrics = vec![Quadric::default(); position_count];
    let vertex_edges: HashSet<(u32, u32)> = triangles
        .iter()
        .flat_map(|&[a, b, c]| [(a, b), (b, c), (c, a)])
        .collect();
    for triangle in &triangles {
        let corners = triangle.map(|vertex| position_of[vertex as usize]);
        let [p0, p1, p2] = corners.map(|position| points[position as usize]);
        let normal = (p1 - p0).cross(p2 - p0);
        let double_area = normal.length();
        if double_area <= 0.0 || double_area.is_nan() {
            continue;
        }
        let normal = normal / double_area;
        let quadric = Quadric::from_plane(normal, p0, double_area * 0.5);
        for position in corners {
            quadrics[position as usize] += quadric;
        }

        for (i, j) in [(0, 1), (1, 2), (2, 0)] {
            if vertex_edges.contains(&(triangle[j], triangle[i])) {
                continue;
            }
            let (a, b) = (corners[i], corners[j]);
            let edge = points[b as usize] - points[a as usize];
            let edge_normal = edge.cross(normal).normalize_or_zero();
            if edge_normal == DVec3::ZERO {
                continue;
            }
            let quadric = Quadric::from_plane(
                edge_normal,
                points[a as usize],
                EDGE_WEIGHT * edge.length_squared(),
            );
            quadrics[a as usize] += quadric;
            quadrics[b as usize] += quadric;
        }
    }
    drop(vertex_edges);

    let max_error_squared = f64::from(max_error) * f64::from(max_error);
    let mut error_squared = 0.0f64;
    let mut vertex_remap: Vec<u32> = (0..positions.len() as u32).collect();

    while triangles.len() > target_triangle_count {
        let position_of_vertex = |vertex: u32| position_of[vertex as usize];

        // The triangles around each position.
        let mut adjacency_offsets = vec![0u32; position_count + 1];
        for triangle in &triangles {
            for &vertex in triangle {
                adjacency_offsets[position_of_vertex(vertex) as usize + 1] += 1;
            }
        }
        for i in 0..position_count {
            adjacency_offsets[i + 1] += adjacency_offsets[i];
        }
        let mut adjacency = vec![0u32; triangles.len() * 3];
        let mut adjacency_fill = adjacency_offsets.clone();
        for (index, triangle) in triangles.iter().enumerate() {
            for &vertex in triangle {
                let fill = &mut adjacency_fill[position_of_vertex(vertex) as usize];
                adjacency[*fill as usize] = index as u32;
                *fill += 1;
            }
        }
        let triangles_around = |position: u32| {
            &adjacency[adjacency_offsets[position as usize] as usize
                ..adjacency_offsets[position as usize + 1] as usize]
        };

        // The positions of the corners of each triangle.
        let triangle_positions: Vec<[u32; 3]> = triangles
            .iter()
            .map(|triangle| triangle.map(position_of_vertex))
            .collect();
        // The position that follows `position` in a triangle.
        let next_position = |triangle: u32, position: u32| {
            let corners = triangle_positions[triangle as usize];
            let i = corners.iter().position(|&corner| corner == position)?;
            Some(corners[(i + 1) % 3])
        };
        // The number of triangles with an edge going from `a` to `b`.
        let half_edge_count = |a: u32, b: u32| {
            triangles_around(a)
                .iter()
                .filter(|&&triangle| next_position(triangle, a) == Some(b))
                .count()
        };
        let has_half_edge = |a: u32, b: u32| {
            triangles_around(a)
                .iter()
                .any(|&triangle| next_position(triangle, a) == Some(b))
        };

        // Classify positions from the edges between them.
        let mut locked = vec![false; position_count];
        let mut border_edges_out = vec![0u32; position_count];
        let mut border_edges_in = vec![0u32; position_count];
        for position in 0..position_count as u32 {
            for &triangle in triangles_around(position) {
                let corners = triangle_positions[triangle as usize];
                let Some(i) = corners.iter().position(|&corner| corner == position) else {
                    continue;
                };
                let (next, previous) = (corners[(i + 1) % 3], corners[(i + 2) % 3]);
                if half_edge_count(position, next) > 1 {
                    // An edge used by more than two triangles.
                    locked[position as usize] = true;
                    locked[next as usize] = true;
                }
                if !has_half_edge(next, position) {
                    border_edges_out[position as usize] += 1;
                }
                if !has_half_edge(position, previous) {
                    border_edges_in[position as usize] += 1;
                }
            }
        }
        // The number of vertices used by each position.
        let mut vertex_seen = vec![false; positions.len()];
        let mut wedge_counts = vec![0u32; position_count];
        for triangle in &triangles {
            for &vertex in triangle {
                if !vertex_seen[vertex as usize] {
                    vertex_seen[vertex as usize] = true;
                    wedge_counts[position_of_vertex(vertex) as usize] += 1;
                }
            }
        }
        drop(vertex_seen);
        let kind = |position: u32| {
            let position = position as usize;
            // Positions where more than two attribute regions meet are kept in place, so that
            // the regions don't bleed into each other.
            if locked[position] || wedge_counts[position] > 2 {
                return PositionKind::Locked;
            }
            match (border_edges_out[position], border_edges_in[position]) {
                (0, 0) => PositionKind::Manifold,
                (1, 1) if !lock_border => PositionKind::Border,
                _ => PositionKind::Locked,
            }
        };
        let is_border_edge = |a: u32, b: u32| has_half_edge(a, b) != has_half_edge(b, a);
        let can_collapse = |from: u32, to: u32| match kind(from) {
            PositionKind::Manifold => true,
            PositionKind::Border => is_border_edge(from, to),
            PositionKind::Locked => false,
        };

        // Find the cheapest way to collapse each edge.
        let mut collapses: Vec<(f64, u32, u32)> = Vec::new();
        let half_edges = (0..triangle_positions.len() as u32).flat_map(|triangle| {
            let [a, b, c] = triangle_positions[triangle as usize];
            [(a, b), (b, c), (c, a)]
        });
        for (a, b) in half_edges {
            // Consider each edge once.
            if a > b && has_half_edge(b, a) {
                continue;
            }
            let quadric = {
                let mut quadric = quadrics[a as usize];
                quadric += quadrics[b as usize];
                quadric
            };
            let collapse_error = |from: u32, to: u32| {
                can_collapse(from, to).then(|| quadric.error(points[to as usize]))
            };
            let collapse = match (collapse_error(a, b), collapse_error(b, a)) {
                (Some(ab), Some(ba)) if ba < ab => Some((ba, b, a)),
                (Some(ab), _) => Some((ab, a, b)),
                (None, Some(ba)) => Some((ba, b, a)),
                (None, None) => None,
            };
            collapses.extend(collapse);
        }
        collapses.sort_unstable_by(|a, b| a.0.total_cmp(&b.0).then((a.1, a.2).cmp(&(b.1, b.2))));

        // Collapse as many edges as possible, cheapest first. Positions around a collapsed edge
        // are not collapsed again in the same pass, so that the checks done for each collapse stay
        // valid.
        let mut used = vec![false; position_count];
        let mut remaining_triangles = triangles.len();
        let mut collapsed = false;
        let mut wedge_mapping: Vec<(u32, u32)> = Vec::new();
        let mut from_vertices: Vec<u32> = Vec::new();
        let mut from_neighbors: Vec<u32> = Vec::new();
        let mut opposite_positions: Vec<u32> = Vec::new();
        for &(collapse_error, from, to) in &collapses {
            if remaining_triangles <= target_triangle_count
                || collapse_error > max_error_squared
                || collapse_error.is_nan()
            {
                break;
            }
            if used[from as usize] || used[to as usize] {
                continue;
            }

            // Check that no triangle flips, and count the triangles that disappear.
            let to_point = points[to as usize];
            let mut removed_triangles = 0;
            let mut flips = false;
            for &triangle in triangles_around(from) {
                let corners = triangles[triangle as usize].map(position_of_vertex);
                if corners.contains(&to) {
                    removed_triangles += 1;
                    continue;
                }
                let [p0, p1, p2] = corners.map(|position| points[position as usize]);
                let normal = (p1 - p0).cross(p2 - p0);
                let [q0, q1, q2] = corners.map(|position| {
                    if position == from {
                        to_point
                    } else {
                        points[position as usize]
                    }
                });
                let new_normal = (q1 - q0).cross(q2 - q0);
                let keeps_orientation = normal.dot(new_normal)
                    > MIN_NORMAL_COSINE * normal.length() * new_normal.length();
                if !keeps_orientation {
                    flips = true;
                    break;
                }
            }
            if flips {
                continue;
            }

            // Keep the surface manifold: `from` and `to` can only share the neighbors on the other
            // side of their common triangles, and no triangle can end up duplicated, as when
            // collapsing a closed tetrahedron into two triangles facing opposite directions.
            from_neighbors.clear();
            opposite_positions.clear();
            for &triangle in triangles_around(from) {
                let corners = triangle_positions[triangle as usize];
                for position in corners {
                    if position != from && !from_neighbors.contains(&position) {
                        from_neighbors.push(position);
                    }
                    if corners.contains(&to) && position != from && position != to {
                        opposite_positions.push(position);
                    }
                }
            }
            let mut manifold = true;
            'triangles: for &triangle in triangles_around(to) {
                let corners = triangle_positions[triangle as usize];
                if corners.contains(&from) {
                    continue;
                }
                for position in corners {
                    if position != to
                        && from_neighbors.contains(&position)
                        && !opposite_positions.contains(&position)
                    {
                        manifold = false;
                        break 'triangles;
                    }
                }
            }
            let sorted = |mut corners: [u32; 3]| {
                corners.sort_unstable();
                corners
            };
            let creates_duplicate = || {
                triangles_around(from).iter().any(|&triangle| {
                    let corners = triangle_positions[triangle as usize];
                    if corners.contains(&to) {
                        return false;
                    }
                    let collapsed = sorted(corners.map(|p| if p == from { to } else { p }));
                    triangles_around(to)
                        .iter()
                        .any(|&other| sorted(triangle_positions[other as usize]) == collapsed)
                })
            };
            if !manifold || creates_duplicate() {
                continue;
            }

            // Each vertex of `from` is merged into the vertex of `to` it shares an edge with, so
            // that attributes stay continuous. This only works if there is exactly one such
            // vertex, and if different vertices of `from` go to different vertices of `to`.
            wedge_mapping.clear();
            from_vertices.clear();
            let mut valid = true;
            'triangles: for &triangle in triangles_around(from) {
                let vertices = triangles[triangle as usize];
                for i in 0..3 {
                    let vertex = vertices[i];
                    if position_of_vertex(vertex) != from {
                        continue;
                    }
                    if !from_vertices.contains(&vertex) {
                        from_vertices.push(vertex);
                    }
                    for other in [vertices[(i + 1) % 3], vertices[(i + 2) % 3]] {
                        if position_of_vertex(other) != to {
                            continue;
                        }
                        match wedge_mapping.iter().find(|&&(v, _)| v == vertex) {
                            Some(&(_, target)) if target != other => {
                                valid = false;
                                break 'triangles;
                            }
                            Some(_) => {}
                            None => wedge_mapping.push((vertex, other)),
                        }
                    }
                }
            }
            let targets_are_distinct = wedge_mapping
                .iter()
                .enumerate()
                .all(|(i, &(_, a))| wedge_mapping[i + 1..].iter().all(|&(_, b)| a != b));
            if !valid || wedge_mapping.len() != from_vertices.len() || !targets_are_distinct {
                continue;
            }

            for &(vertex, target) in &wedge_mapping {
                vertex_remap[vertex as usize] = target;
            }
            let from_quadric = quadrics[from as usize];
            quadrics[to as usize] += from_quadric;
            for &triangle in triangles_around(from) {
                for vertex in triangles[triangle as usize] {
                    used[position_of_vertex(vertex) as usize] = true;
                }
            }
            error_squared = error_squared.max(collapse_error);
            remaining_triangles -= removed_triangles;
            collapsed = true;
        }
        drop(adjacency);

        if !collapsed {
            break;
        }

        // Apply the collapses, and remove the triangles that became degenerate.
        triangles.retain_mut(|triangle| {
            *triangle = triangle.map(|vertex| vertex_remap[vertex as usize]);
            let [a, b, c] = triangle.map(|vertex| position_of[vertex as usize]);
            a != b && b != c && c != a
        });
    }

    indices.clear();
    indices.extend(triangles.iter().flatten());
    error_squared.sqrt() as f32
}

#[cfg(test)]
mod tests {
    use super::MeshSimplificationSettings;
    use crate::{
        Indices, Mesh, MeshBuilder, MeshOptimizationError, Meshable, PrimitiveTopology,
        VertexAttributeValues,
    };
    use alloc::vec::Vec;
    use bevy_asset::RenderAssetUsages;
    use bevy_math::Vec3;
    use bevy_shape::{Sphere, Torus};

    /// A flat `size` x `size` grid of quads in the XY plane, with UVs.
    fn grid(size: u32) -> Mesh {
        let row = size + 1;
        let positions: Vec<[f32; 3]> = (0..row * row)
            .map(|i| [(i % row) as f32, (i / row) as f32, 0.0])
            .collect();
        let uvs: Vec<[f32; 2]> = positions.iter().map(|p| [p[0], p[1]]).collect();
        let mut indices = Vec::new();
        for y in 0..size {
            for x in 0..size {
                let i = y * row + x;
                indices.extend([i, i + 1, i + row, i + 1, i + row + 1, i + row]);
            }
        }
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
    }

    fn positions(mesh: &Mesh) -> &[[f32; 3]] {
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("missing positions");
        };
        positions
    }

    fn triangles(mesh: &Mesh) -> Vec<[Vec3; 3]> {
        let positions = positions(mesh);
        let indices: Vec<usize> = mesh.indices().unwrap().iter().collect();
        indices
            .as_chunks::<3>()
            .0
            .iter()
            .map(|triangle| triangle.map(|i| Vec3::from(positions[i])))
            .collect()
    }

    fn area(mesh: &Mesh) -> f32 {
        triangles(mesh)
            .iter()
            .map(|[a, b, c]| (b - a).cross(c - a).length() * 0.5)
            .sum()
    }

    fn settings(target_ratio: f32, max_error: f32) -> MeshSimplificationSettings {
        MeshSimplificationSettings {
            target_ratio,
            max_error,
            lock_border: false,
        }
    }

    #[test]
    fn flat_grid_simplifies_to_two_triangles() {
        let mut mesh = grid(16);
        let error = mesh.simplify(&settings(0.0, 1e-4)).unwrap();

        // The corners can't move without changing the shape, but everything else can go.
        assert_eq!(mesh.indices().unwrap().len(), 6);
        assert_eq!(mesh.count_vertices(), 4);
        assert!(error < 1e-4, "{error}");
        assert!((area(&mesh) - 256.0).abs() < 1e-3);
        // Triangles still face the same direction.
        for [a, b, c] in triangles(&mesh) {
            assert!((b - a).cross(c - a).z > 0.0);
        }
    }

    #[test]
    fn locked_border_stays_in_place() {
        let mut mesh = grid(8);
        mesh.simplify(&MeshSimplificationSettings {
            target_ratio: 0.0,
            max_error: f32::INFINITY,
            lock_border: true,
        })
        .unwrap();

        // The 32 border vertices remain, the interior ones can be removed.
        let positions = positions(&mesh);
        let border = positions
            .iter()
            .filter(|p| p[0] == 0.0 || p[0] == 8.0 || p[1] == 0.0 || p[1] == 8.0)
            .count();
        assert_eq!(border, 32);
        assert!(positions.len() < 81);
        assert!((area(&mesh) - 64.0).abs() < 1e-3);
    }

    #[test]
    fn sphere_keeps_its_shape() {
        let mut mesh = Sphere::new(1.0).mesh().uv(64, 32);
        let triangle_count = mesh.indices().unwrap().len() / 3;
        let error = mesh.simplify(&settings(0.2, 0.02)).unwrap();

        let simplified_triangles = triangles(&mesh);
        assert!(simplified_triangles.len() <= triangle_count / 5 + 1);
        assert!(simplified_triangles.len() > 50);
        assert!(error <= 0.02, "{error}");
        for [a, b, c] in simplified_triangles {
            // The triangles stay close to the sphere, and face outwards.
            let center = (a + b + c) / 3.0;
            assert!(center.length() > 0.9, "{center}");
            assert!((b - a).cross(c - a).dot(center) > 0.0);
        }
    }

    #[test]
    fn max_error_is_respected() {
        let original = Torus::default().mesh().build();
        let triangle_count = original.indices().unwrap().len() / 3;
        let mut previous_triangle_count = triangle_count;
        for max_error in [0.001, 0.01, 0.05, 0.2] {
            let mut mesh = original.clone();
            let error = mesh.simplify(&settings(0.0, max_error)).unwrap();
            let simplified_triangle_count = mesh.indices().unwrap().len() / 3;
            assert!(error <= max_error, "{error} > {max_error}");
            // Allowing more error removes more triangles.
            assert!(simplified_triangle_count <= previous_triangle_count);
            previous_triangle_count = simplified_triangle_count;
        }
        assert!(previous_triangle_count < triangle_count / 4);
    }

    #[test]
    fn uv_seams_are_preserved() {
        // Two UV islands: the left and right halves of a grid, with separate vertices on the
        // column where they meet.
        let size = 8u32;
        let mut positions = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();
        for island in 0..2u32 {
            let base = positions.len() as u32;
            let columns = size / 2 + 1;
            for y in 0..=size {
                for x in 0..columns {
                    let x = x + island * size / 2;
                    positions.push([x as f32, y as f32, 0.0]);
                    uvs.push([island as f32 * 10.0 + x as f32, y as f32]);
                }
            }
            for y in 0..size {
                for x in 0..columns - 1 {
                    let i = base + y * columns + x;
                    indices.extend([i, i + 1, i + columns, i + 1, i + columns + 1, i + columns]);
                }
            }
        }
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U16(indices.iter().map(|&i| i as u16).collect()));

        mesh.simplify(&settings(0.0, 1e-4)).unwrap();

        // Each triangle only uses vertices of a single island.
        let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!("missing UVs");
        };
        let indices: Vec<usize> = mesh.indices().unwrap().iter().collect();
        for triangle in indices.as_chunks::<3>().0 {
            let islands = triangle.map(|i| uvs[i][0] >= 10.0);
            assert!(islands.iter().all(|&island| island == islands[0]));
        }
        // The seam is straight, so the islands can be simplified down to their corners.
        assert_eq!(indices.len(), 12);
        assert!(matches!(mesh.indices(), Some(Indices::U16(_))));
        assert!((area(&mesh) - 64.0).abs() < 1e-3);
    }

    #[test]
    fn complex_meshes_do_not_break() {
        // A triangle soup with non-manifold edges, degenerate triangles and duplicate
        // triangles.
        let mut state = 7u32;
        let mut random = || {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            state >> 8
        };
        let positions: Vec<[f32; 3]> = (0..200)
            .map(|_| [random() % 10, random() % 10, random() % 10].map(|c| c as f32))
            .collect();
        let indices: Vec<u32> = (0..3000).map(|_| random() % 200).collect();
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_indices(Indices::U32(indices));
        mesh.simplify(&settings(0.1, f32::INFINITY)).unwrap();
        assert!(mesh.indices().unwrap().len().is_multiple_of(3));
    }

    #[test]
    fn closed_meshes_do_not_vanish() {
        // A closed tetrahedron, and a closed cube with shared vertices.
        let tetrahedron = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
        )
        .with_inserted_indices(Indices::U32(vec![0, 2, 1, 0, 1, 3, 0, 3, 2, 1, 2, 3]));
        let cube_positions: Vec<[f32; 3]> = (0..8)
            .map(|i| [(i & 1) as f32, ((i >> 1) & 1) as f32, ((i >> 2) & 1) as f32])
            .collect();
        let cube = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, cube_positions)
        .with_inserted_indices(Indices::U32(vec![
            0, 2, 1, 1, 2, 3, 4, 5, 6, 5, 7, 6, 0, 1, 4, 1, 5, 4, 2, 6, 3, 3, 6, 7, 0, 4, 2, 2, 4,
            6, 1, 3, 5, 3, 7, 5,
        ]));

        for mesh in [tetrahedron, cube] {
            let mut mesh = mesh;
            mesh.simplify(&settings(0.0, f32::INFINITY)).unwrap();
            let triangles = triangles(&mesh);
            // At least a tetrahedron remains, without duplicated triangles.
            assert!(triangles.len() >= 4, "{triangles:?}");
            let mut sorted: Vec<[[u32; 3]; 3]> = triangles
                .iter()
                .map(|triangle| {
                    let mut corners = triangle.map(|p| p.to_array().map(f32::to_bits));
                    corners.sort_unstable();
                    corners
                })
                .collect();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), triangles.len());
        }
    }

    #[test]
    fn simplify_errors() {
        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        );
        assert!(matches!(
            mesh.clone().simplified(&Default::default()),
            Err(MeshOptimizationError::MissingIndices)
        ));
        assert!(matches!(
            mesh.clone()
                .with_inserted_indices(Indices::U32(vec![0, 1, 2]))
                .compressed_mesh(&crate::MeshCompressionArgs::regular())
                .unwrap()
                .simplified(&Default::default()),
            Err(MeshOptimizationError::UnsupportedPositions)
        ));
        assert!(matches!(
            Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default())
                .simplified(&Default::default()),
            Err(MeshOptimizationError::UnsupportedTopology(
                PrimitiveTopology::LineList
            ))
        ));
        let triangle = mesh.with_inserted_indices(Indices::U32(vec![0, 1, 2]));
        for invalid_settings in [
            settings(f32::NAN, 0.01),
            settings(0.5, f32::NAN),
            settings(0.5, -0.01),
        ] {
            assert!(matches!(
                triangle.clone().simplified(&invalid_settings),
                Err(MeshOptimizationError::InvalidSimplificationSettings(_))
            ));
        }
    }
}
