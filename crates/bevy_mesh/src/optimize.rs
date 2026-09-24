//! Reordering of mesh data to make rendering cheaper for the GPU.
//!
//! The order of the triangles in the index buffer and of the vertices in the vertex buffer
//! doesn't change what a mesh looks like, but it has a large impact on how fast it renders:
//!
//! - GPUs keep the results of the vertex shader for recently processed vertices in a small
//!   "post-transform" cache. Drawing triangles that share vertices one after the other means that
//!   shared vertices are only transformed once, instead of up to six times for a regular grid.
//!   [`Mesh::optimize_vertex_cache`] reorders triangles to take advantage of this cache.
//! - Vertex attributes are fetched from memory in cache lines. Storing vertices in the order in
//!   which they are first used means that neighboring vertices are fetched together, instead of
//!   the GPU jumping around the vertex buffer. [`Mesh::optimize_vertex_fetch`] reorders vertices
//!   in that way (and drops the vertices that aren't referenced by any primitive).
//!
//! [`Mesh::optimize_for_gpu`] applies both optimizations in the right order. The effect of the
//! optimizations can be measured with [`Mesh::analyze_vertex_cache`] and
//! [`Mesh::analyze_vertex_fetch`].
//!
//! The triangle reordering is an implementation of Tom Forsyth's
//! [Linear-Speed Vertex Cache Optimization](https://tomforsyth1000.github.io/papers/fast_vert_cache_opt.html),
//! the same family of algorithms that is used by tools like
//! [meshoptimizer](https://github.com/zeux/meshoptimizer).

use crate::{Indices, Mesh, MeshAccessError, PrimitiveTopology};
use alloc::vec::Vec;
use bevy_math::ops;
use thiserror::Error;

/// Size of the LRU cache simulated while reordering triangles.
///
/// Forsyth's algorithm isn't very sensitive to the exact cache size, and a size of 32
/// gives good results on a wide range of GPUs.
const SIMULATED_CACHE_SIZE: usize = 32;

/// Number of live triangles above which a vertex no longer gets a lower score.
const MAX_SCORED_VALENCE: usize = 32;

/// How fast the score of a cached vertex decays with its age.
const CACHE_DECAY_POWER: f32 = 1.5;

/// Score of the vertices of the most recently emitted triangle.
///
/// This is deliberately lower than the score of vertices that are slightly older, so that the
/// optimizer doesn't create long strips (which only reuse two vertices per triangle) and instead
/// "fans out" around vertices.
const LAST_TRIANGLE_SCORE: f32 = 0.75;

/// Boost given to vertices with few remaining triangles, so that the optimizer finishes
/// the triangles around a vertex instead of leaving lone triangles behind.
const VALENCE_BOOST_SCALE: f32 = 2.0;

/// See [`VALENCE_BOOST_SCALE`].
const VALENCE_BOOST_POWER: f32 = 0.5;

/// Size of a cache line in [`Mesh::analyze_vertex_fetch`].
const FETCH_CACHE_LINE_SIZE: usize = 64;

/// Number of cache lines in the direct-mapped cache simulated by [`Mesh::analyze_vertex_fetch`].
const FETCH_CACHE_LINE_COUNT: usize = 16 * 1024 / FETCH_CACHE_LINE_SIZE;

/// An error that occurred while optimizing or analyzing a [`Mesh`].
#[derive(Error, Debug, Clone)]
pub enum MeshOptimizationError {
    /// The operation requires a [`PrimitiveTopology::TriangleList`] mesh.
    #[error("Expected a mesh with `PrimitiveTopology::TriangleList`, got `{0:?}`")]
    UnsupportedTopology(PrimitiveTopology),
    /// The mesh doesn't have [`Indices`].
    ///
    /// Non-indexed meshes don't share any vertex between primitives, so there is nothing to
    /// optimize. [`Mesh::merge_duplicate_vertices`] can be used to create indices first.
    #[error("The mesh doesn't have indices")]
    MissingIndices,
    /// The number of indices is not a multiple of 3 for a [`PrimitiveTopology::TriangleList`] mesh.
    #[error("The index count ({0}) of a triangle list must be a multiple of 3")]
    AbruptIndicesEnd(usize),
    /// An index references a vertex that doesn't exist.
    #[error("Index {index} is out of bounds for a mesh with {vertex_count} vertices")]
    IndexOutOfBounds {
        /// The invalid index.
        index: usize,
        /// The number of vertices of the mesh.
        vertex_count: usize,
    },
    /// The mesh doesn't have [`Mesh::ATTRIBUTE_POSITION`] in the [`VertexFormat::Float32x3`](crate::VertexFormat::Float32x3)
    /// format, for example because its positions have been compressed.
    #[error(
        "The mesh must have `Mesh::ATTRIBUTE_POSITION` with the `VertexFormat::Float32x3` format"
    )]
    UnsupportedPositions,
    /// The mesh data has been extracted to the `RenderWorld`.
    #[error("Mesh access error: {0}")]
    MeshAccessError(#[from] MeshAccessError),
}

/// Statistics about the use of the post-transform vertex cache by a mesh, as returned by
/// [`Mesh::analyze_vertex_cache`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VertexCacheStatistics {
    /// The number of times the vertex shader runs when drawing the mesh.
    pub vertices_transformed: usize,
    /// The number of distinct vertices referenced by the indices of the mesh.
    pub unique_vertices: usize,
    /// Average cache miss ratio: the number of transformed vertices per triangle.
    ///
    /// Lower is better. This is 3.0 when there is no reuse at all, and can get close to 0.5 for
    /// large regular grids, where each vertex is shared by 6 triangles.
    pub acmr: f32,
    /// Average transformed vertex ratio: the number of times each vertex is transformed on average.
    ///
    /// Lower is better, 1.0 is optimal.
    pub atvr: f32,
}

/// Statistics about the memory traffic generated by fetching the vertices of a mesh, as returned
/// by [`Mesh::analyze_vertex_fetch`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VertexFetchStatistics {
    /// The estimated number of bytes read from memory to fetch the vertex attributes.
    pub bytes_fetched: usize,
    /// The ratio between [`bytes_fetched`](Self::bytes_fetched) and the size of the vertex data
    /// that is referenced by the indices.
    ///
    /// Lower is better, and 1.0 means that each referenced byte is only fetched once.
    pub overfetch: f32,
}

impl Mesh {
    /// Reorders the triangles of this mesh so that the GPU transforms as few vertices as possible,
    /// by taking advantage of its post-transform vertex cache.
    ///
    /// This only changes the order of the indices, the vertex data is left untouched. The winding
    /// order of each triangle is preserved. This is most effective when followed by
    /// [`Mesh::optimize_vertex_fetch`], see [`Mesh::optimize_for_gpu`].
    ///
    /// Returns an error if the mesh isn't an indexed [`PrimitiveTopology::TriangleList`], if its
    /// indices are invalid, or if the mesh data has been extracted to the `RenderWorld`.
    pub fn optimize_vertex_cache(&mut self) -> Result<(), MeshOptimizationError> {
        if self.primitive_topology() != PrimitiveTopology::TriangleList {
            return Err(MeshOptimizationError::UnsupportedTopology(
                self.primitive_topology(),
            ));
        }
        let vertex_count = self.try_count_vertices()?;
        let topology = self.primitive_topology();
        let indices = self
            .try_indices_mut_option()?
            .ok_or(MeshOptimizationError::MissingIndices)?;
        validate_indices(indices, vertex_count, topology)?;

        with_u32_indices(indices, |indices| {
            optimize_vertex_cache(indices, vertex_count);
        });

        Ok(())
    }

    /// Reorders the vertices of this mesh in the order in which they are first used by the
    /// indices, and removes the vertices that aren't used by any primitive.
    ///
    /// This makes the memory accesses of the GPU more coherent when fetching vertex attributes.
    /// The indices, all the vertex attributes and the morph targets are updated accordingly. The
    /// order of the primitives is left untouched, so this should be done after reordering
    /// triangles with [`Mesh::optimize_vertex_cache`], see [`Mesh::optimize_for_gpu`].
    ///
    /// This works for all primitive topologies. For strip topologies, primitive restart values
    /// (the maximum value of the index format) are preserved.
    ///
    /// Note that per-vertex data stored outside of the mesh (for example in a storage buffer
    /// indexed with the vertex index in a shader) isn't updated.
    ///
    /// Returns an error if the mesh doesn't have indices, if its indices are invalid,
    /// or if the mesh data has been extracted to the `RenderWorld`.
    pub fn optimize_vertex_fetch(&mut self) -> Result<(), MeshOptimizationError> {
        let vertex_count = self.try_count_vertices()?;
        let topology = self.primitive_topology();
        let indices = self
            .try_indices_mut_option()?
            .ok_or(MeshOptimizationError::MissingIndices)?;
        validate_indices(indices, vertex_count, topology)?;

        let restart_index = restart_index(indices, topology);
        let new_to_old = with_u32_indices(indices, |indices| {
            remap_vertices_by_first_use(indices, vertex_count, restart_index)
        });

        let is_identity = new_to_old.len() == vertex_count
            && new_to_old
                .iter()
                .enumerate()
                .all(|(new, &old)| new == old as usize);
        if !is_identity {
            self.try_gather_vertices(|| new_to_old.iter().map(|&old| old as usize))?;
        }

        Ok(())
    }

    /// Makes this mesh cheaper to render by reordering its triangles with
    /// [`Mesh::optimize_vertex_cache`], and then its vertices with [`Mesh::optimize_vertex_fetch`].
    ///
    /// This doesn't change the appearance of the mesh. This is a relatively cheap operation
    /// (linear in the number of triangles), that is best done once when loading or generating
    /// a mesh, as most authoring tools don't order mesh data optimally.
    ///
    /// Triangle reordering is only done for [`PrimitiveTopology::TriangleList`] meshes; for other
    /// topologies only the vertices are reordered.
    ///
    /// Returns an error if the mesh doesn't have indices, if its indices are invalid,
    /// or if the mesh data has been extracted to the `RenderWorld`.
    ///
    /// ```
    /// # use bevy_mesh::{Mesh, Meshable};
    /// # use bevy_shape::Sphere;
    /// let mut mesh = Sphere::new(1.0).mesh().uv(32, 18);
    ///
    /// let before = mesh.analyze_vertex_cache(16).unwrap();
    /// mesh.optimize_for_gpu().unwrap();
    /// let after = mesh.analyze_vertex_cache(16).unwrap();
    ///
    /// // Fewer vertex shader invocations are needed to draw the same mesh.
    /// assert!(after.vertices_transformed < before.vertices_transformed);
    /// ```
    pub fn optimize_for_gpu(&mut self) -> Result<(), MeshOptimizationError> {
        if self.primitive_topology() == PrimitiveTopology::TriangleList {
            self.optimize_vertex_cache()?;
        }
        self.optimize_vertex_fetch()
    }

    /// Consumes the mesh and returns a mesh optimized with [`Mesh::optimize_for_gpu`].
    ///
    /// Returns an error if the mesh doesn't have indices, if its indices are invalid,
    /// or if the mesh data has been extracted to the `RenderWorld`.
    pub fn optimized_for_gpu(mut self) -> Result<Self, MeshOptimizationError> {
        self.optimize_for_gpu()?;
        Ok(self)
    }

    /// Estimates how efficiently this mesh uses the post-transform vertex cache of the GPU, by
    /// simulating a first-in first-out cache of `cache_size` vertices.
    ///
    /// Actual GPUs process vertices in batches rather than through a simple FIFO cache, but this
    /// model is accurate enough to compare different orderings of the same mesh. A `cache_size`
    /// between 16 and 32 is representative of most hardware.
    ///
    /// Returns an error if the mesh isn't an indexed [`PrimitiveTopology::TriangleList`], if its
    /// indices are invalid, or if the mesh data has been extracted to the `RenderWorld`.
    pub fn analyze_vertex_cache(
        &self,
        cache_size: usize,
    ) -> Result<VertexCacheStatistics, MeshOptimizationError> {
        if self.primitive_topology() != PrimitiveTopology::TriangleList {
            return Err(MeshOptimizationError::UnsupportedTopology(
                self.primitive_topology(),
            ));
        }
        let vertex_count = self.try_count_vertices()?;
        let indices = self
            .try_indices_option()?
            .ok_or(MeshOptimizationError::MissingIndices)?;
        validate_indices(indices, vertex_count, self.primitive_topology())?;

        Ok(analyze_vertex_cache(
            indices.iter(),
            indices.len() / 3,
            vertex_count,
            cache_size,
        ))
    }

    /// Estimates the amount of memory traffic needed to fetch the vertex attributes of this mesh,
    /// by simulating a small cache in front of the interleaved vertex buffer that Bevy uploads
    /// to the GPU (see [`Mesh::get_vertex_size`]).
    ///
    /// This is only a rough model of the memory hierarchy of a GPU, which is useful to compare
    /// different orderings of the same mesh.
    ///
    /// Returns an error if the mesh doesn't have indices, if its indices are invalid,
    /// or if the mesh data has been extracted to the `RenderWorld`.
    pub fn analyze_vertex_fetch(&self) -> Result<VertexFetchStatistics, MeshOptimizationError> {
        let vertex_count = self.try_count_vertices()?;
        let indices = self
            .try_indices_option()?
            .ok_or(MeshOptimizationError::MissingIndices)?;
        validate_indices(indices, vertex_count, self.primitive_topology())?;

        let restart_index = restart_index(indices, self.primitive_topology());
        Ok(analyze_vertex_fetch(
            indices
                .iter()
                .filter(|&index| Some(index) != restart_index.map(|restart| restart as usize)),
            vertex_count,
            self.get_vertex_size() as usize,
        ))
    }

    /// Like [`Mesh::count_vertices`], but returns an error instead of panicking if the mesh data
    /// has been extracted to the `RenderWorld`.
    pub(crate) fn try_count_vertices(&self) -> Result<usize, MeshAccessError> {
        Ok(self
            .try_attributes()?
            .map(|(_, values)| values.len())
            .min()
            .unwrap_or(0))
    }
}

/// Returns the primitive restart value of `indices`, if primitive restart applies to `topology`.
fn restart_index(indices: &Indices, topology: PrimitiveTopology) -> Option<u32> {
    if !topology.is_strip() {
        return None;
    }
    Some(match indices {
        Indices::U16(_) => u16::MAX as u32,
        Indices::U32(_) => u32::MAX,
    })
}

/// Checks that the indices are valid for a mesh with `vertex_count` vertices and the given
/// `topology`.
pub(crate) fn validate_indices(
    indices: &Indices,
    vertex_count: usize,
    topology: PrimitiveTopology,
) -> Result<(), MeshOptimizationError> {
    if topology == PrimitiveTopology::TriangleList && !indices.len().is_multiple_of(3) {
        return Err(MeshOptimizationError::AbruptIndicesEnd(indices.len()));
    }
    let restart_index = restart_index(indices, topology).map(|restart| restart as usize);
    match indices
        .iter()
        .find(|&index| index >= vertex_count && Some(index) != restart_index)
    {
        Some(index) => Err(MeshOptimizationError::IndexOutOfBounds {
            index,
            vertex_count,
        }),
        None => Ok(()),
    }
}

/// Runs `f` on the indices as `u32`, converting them back and forth if they are stored as `u16`.
///
/// `f` must not produce values that don't fit in the original index format.
pub(crate) fn with_u32_indices<R>(indices: &mut Indices, f: impl FnOnce(&mut [u32]) -> R) -> R {
    match indices {
        Indices::U32(indices) => f(indices),
        Indices::U16(narrow_indices) => {
            let mut wide_indices: Vec<u32> =
                narrow_indices.iter().copied().map(u32::from).collect();
            let result = f(&mut wide_indices);
            for (narrow, wide) in narrow_indices.iter_mut().zip(wide_indices) {
                *narrow = wide as u16;
            }
            result
        }
    }
}

/// Precomputed vertex scores for Forsyth's algorithm.
struct VertexScoreTable {
    /// Score of a vertex, indexed by its position in the simulated LRU cache.
    cache: [f32; SIMULATED_CACHE_SIZE],
    /// Score of a vertex, indexed by its number of live triangles (clamped).
    valence: [f32; MAX_SCORED_VALENCE + 1],
}

impl VertexScoreTable {
    fn new() -> Self {
        let mut cache = [0.0; SIMULATED_CACHE_SIZE];
        for (position, score) in cache.iter_mut().enumerate() {
            *score = if position < 3 {
                LAST_TRIANGLE_SCORE
            } else {
                let scale = 1.0 / (SIMULATED_CACHE_SIZE - 3) as f32;
                ops::powf(1.0 - (position - 3) as f32 * scale, CACHE_DECAY_POWER)
            };
        }

        let mut valence = [0.0; MAX_SCORED_VALENCE + 1];
        for (live_triangles, score) in valence.iter_mut().enumerate().skip(1) {
            *score = VALENCE_BOOST_SCALE * ops::powf(live_triangles as f32, -VALENCE_BOOST_POWER);
        }

        Self { cache, valence }
    }

    /// The score of a vertex at `cache_position` in the cache (if any) with `live_triangles`
    /// triangles left to emit.
    #[inline]
    fn score(&self, cache_position: Option<usize>, live_triangles: u32) -> f32 {
        if live_triangles == 0 {
            // The score of a vertex without live triangles doesn't matter, as it isn't used to
            // score any triangle.
            return 0.0;
        }
        let cache_score = cache_position.map_or(0.0, |position| self.cache[position]);
        cache_score + self.valence[(live_triangles as usize).min(MAX_SCORED_VALENCE)]
    }
}

/// Reorders the triangles of a triangle list to improve the post-transform vertex cache hit rate,
/// using Tom Forsyth's "Linear-Speed Vertex Cache Optimization" algorithm.
///
/// The indices must be valid for `vertex_count` vertices, and their count must be a multiple of 3.
fn optimize_vertex_cache(indices: &mut [u32], vertex_count: usize) {
    let triangle_count = indices.len() / 3;
    if triangle_count <= 1 {
        return;
    }
    let table = VertexScoreTable::new();

    // Number of triangles that still need to be emitted, for each vertex.
    let mut live_triangles = vec![0u32; vertex_count];
    for &index in indices.iter() {
        live_triangles[index as usize] += 1;
    }

    // For each vertex, the list of triangles that use it, stored contiguously. The live triangles
    // of vertex `v` are `adjacency[adjacency_offsets[v]..][..live_triangles[v]]`: when a triangle
    // is emitted, it is swapped out of this range.
    let mut adjacency_offsets = Vec::with_capacity(vertex_count);
    let mut offset = 0;
    for &count in &live_triangles {
        adjacency_offsets.push(offset);
        offset += count as usize;
    }
    let mut adjacency = vec![0u32; indices.len()];
    let mut adjacency_fill = adjacency_offsets.clone();
    for (triangle, corners) in indices.as_chunks::<3>().0.iter().enumerate() {
        for &vertex in corners {
            adjacency[adjacency_fill[vertex as usize]] = triangle as u32;
            adjacency_fill[vertex as usize] += 1;
        }
    }
    drop(adjacency_fill);

    let mut vertex_scores: Vec<f32> = live_triangles
        .iter()
        .map(|&live| table.score(None, live))
        .collect();
    // A triangle's score is the sum of its vertices' scores. A degenerate triangle counts a repeated
    // vertex several times, which is consistent with it appearing several times in that vertex's
    // adjacency list.
    let mut triangle_scores: Vec<f32> = indices
        .as_chunks::<3>()
        .0
        .iter()
        .map(|corners| {
            corners
                .iter()
                .map(|&vertex| vertex_scores[vertex as usize])
                .sum()
        })
        .collect();
    let mut emitted = vec![false; triangle_count];

    // The simulated LRU cache, most recently used vertex first. It holds up to 3 extra vertices
    // while being updated, before they get evicted.
    let mut cache = [0u32; SIMULATED_CACHE_SIZE + 3];
    let mut cache_len = 0;
    let mut new_cache = [0u32; SIMULATED_CACHE_SIZE + 3];

    // Vertices of emitted triangles, used to find a nearby triangle when the cache doesn't contain
    // any vertex with live triangles.
    let mut dead_end_stack: Vec<u32> = Vec::new();
    // All the triangles before this one have been emitted.
    let mut next_unemitted_triangle = 0;

    let mut output = Vec::with_capacity(indices.len());

    // Start with the triangle that has the best score, which is on the border of the mesh.
    let mut current_triangle = Some(
        triangle_scores
            .iter()
            .enumerate()
            .fold((0, f32::NEG_INFINITY), |best, (triangle, &score)| {
                if score > best.1 {
                    (triangle, score)
                } else {
                    best
                }
            })
            .0,
    );

    while let Some(triangle) = current_triangle {
        let corners = indices.as_chunks::<3>().0[triangle];
        output.extend_from_slice(&corners);
        emitted[triangle] = true;
        dead_end_stack.extend_from_slice(&corners);

        // Remove the triangle from the live triangles of its vertices.
        for vertex in corners {
            let vertex = vertex as usize;
            let live =
                &mut adjacency[adjacency_offsets[vertex]..][..live_triangles[vertex] as usize];
            if let Some(position) = live.iter().position(|&t| t as usize == triangle) {
                live.swap(position, live.len() - 1);
                live_triangles[vertex] -= 1;
            }
        }

        // Move the vertices of the triangle to the front of the cache.
        let mut new_cache_len = 0;
        for vertex in corners {
            if !new_cache[..new_cache_len].contains(&vertex) {
                new_cache[new_cache_len] = vertex;
                new_cache_len += 1;
            }
        }
        for &vertex in &cache[..cache_len] {
            if !corners.contains(&vertex) {
                new_cache[new_cache_len] = vertex;
                new_cache_len += 1;
            }
        }

        // Update the scores of the vertices in the cache, including the ones that just got
        // evicted, and propagate the changes to their live triangles. At the same time, pick the
        // live triangle with the best score as the next one. Some triangles are compared before
        // all their vertices have been updated: this makes the choice slightly less accurate, but
        // halves the cost of each step.
        current_triangle = None;
        let mut best_score = f32::NEG_INFINITY;
        for (position, &vertex) in new_cache[..new_cache_len].iter().enumerate() {
            let vertex = vertex as usize;
            let cache_position = (position < SIMULATED_CACHE_SIZE).then_some(position);
            let score = table.score(cache_position, live_triangles[vertex]);
            let delta = score - vertex_scores[vertex];
            vertex_scores[vertex] = score;
            for &adjacent in
                &adjacency[adjacency_offsets[vertex]..][..live_triangles[vertex] as usize]
            {
                let triangle_score = &mut triangle_scores[adjacent as usize];
                *triangle_score += delta;
                if *triangle_score > best_score {
                    best_score = *triangle_score;
                    current_triangle = Some(adjacent as usize);
                }
            }
        }

        cache_len = new_cache_len.min(SIMULATED_CACHE_SIZE);
        cache[..cache_len].copy_from_slice(&new_cache[..cache_len]);

        if current_triangle.is_none() {
            // Dead end: continue with a triangle adjacent to a recently used vertex, or with the
            // next triangle in the original order.
            while let Some(vertex) = dead_end_stack.pop() {
                let vertex = vertex as usize;
                let live =
                    &adjacency[adjacency_offsets[vertex]..][..live_triangles[vertex] as usize];
                if let Some(&best) = live.iter().max_by(|&&a, &&b| {
                    triangle_scores[a as usize].total_cmp(&triangle_scores[b as usize])
                }) {
                    current_triangle = Some(best as usize);
                    break;
                }
            }
            if current_triangle.is_none() {
                while next_unemitted_triangle < triangle_count && emitted[next_unemitted_triangle] {
                    next_unemitted_triangle += 1;
                }
                if next_unemitted_triangle < triangle_count {
                    current_triangle = Some(next_unemitted_triangle);
                }
            }
        }
    }

    debug_assert_eq!(output.len(), indices.len());
    indices.copy_from_slice(&output);
}

/// Remaps the indices so that vertices are numbered in the order in which they are first used.
///
/// Returns the old index of each new vertex. Vertices that aren't used by any index are dropped.
/// Indices equal to `restart_index` are left untouched.
fn remap_vertices_by_first_use(
    indices: &mut [u32],
    vertex_count: usize,
    restart_index: Option<u32>,
) -> Vec<u32> {
    let mut old_to_new = vec![u32::MAX; vertex_count];
    let mut new_to_old = Vec::with_capacity(vertex_count);
    for index in indices.iter_mut() {
        if Some(*index) == restart_index {
            continue;
        }
        let new_index = &mut old_to_new[*index as usize];
        if *new_index == u32::MAX {
            *new_index = new_to_old.len() as u32;
            new_to_old.push(*index);
        }
        *index = *new_index;
    }
    new_to_old
}

/// Simulates a FIFO post-transform vertex cache of `cache_size` vertices.
fn analyze_vertex_cache(
    indices: impl Iterator<Item = usize>,
    triangle_count: usize,
    vertex_count: usize,
    cache_size: usize,
) -> VertexCacheStatistics {
    // A cache that can hold all the vertices behaves like an infinite cache.
    let cache_size = cache_size.min(vertex_count);
    // The time at which each vertex was last inserted in the cache. A vertex is in the cache if
    // fewer than `cache_size` vertices were inserted after it.
    let mut insertion_times = vec![0usize; vertex_count];
    let mut time = cache_size + 1;
    let mut vertices_transformed = 0;
    let mut unique_vertices = 0;

    for index in indices {
        if time - insertion_times[index] > cache_size {
            if insertion_times[index] == 0 {
                unique_vertices += 1;
            }
            insertion_times[index] = time;
            time += 1;
            vertices_transformed += 1;
        }
    }

    VertexCacheStatistics {
        vertices_transformed,
        unique_vertices,
        acmr: if triangle_count == 0 {
            0.0
        } else {
            vertices_transformed as f32 / triangle_count as f32
        },
        atvr: if unique_vertices == 0 {
            0.0
        } else {
            vertices_transformed as f32 / unique_vertices as f32
        },
    }
}

/// Simulates fetching vertices of `vertex_size` bytes through a direct-mapped cache.
fn analyze_vertex_fetch(
    indices: impl Iterator<Item = usize>,
    vertex_count: usize,
    vertex_size: usize,
) -> VertexFetchStatistics {
    // Tag of the memory line stored in each cache line, plus one so that 0 means empty.
    let mut cache_tags = vec![0usize; FETCH_CACHE_LINE_COUNT];
    let mut referenced = vec![false; vertex_count];
    let mut bytes_fetched = 0;

    for index in indices {
        referenced[index] = true;
        let start = index * vertex_size;
        let end = start + vertex_size;
        for line in start / FETCH_CACHE_LINE_SIZE..end.div_ceil(FETCH_CACHE_LINE_SIZE) {
            let cache_tag = &mut cache_tags[line % FETCH_CACHE_LINE_COUNT];
            if *cache_tag != line + 1 {
                *cache_tag = line + 1;
                bytes_fetched += FETCH_CACHE_LINE_SIZE;
            }
        }
    }

    let referenced_bytes = referenced.iter().filter(|&&r| r).count() * vertex_size;
    VertexFetchStatistics {
        bytes_fetched,
        overfetch: if referenced_bytes == 0 {
            0.0
        } else {
            bytes_fetched as f32 / referenced_bytes as f32
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{MeshOptimizationError, VertexCacheStatistics};
    use crate::{Indices, Mesh, MeshBuilder, Meshable, PrimitiveTopology, VertexAttributeValues};
    use alloc::vec::Vec;
    use bevy_asset::RenderAssetUsages;
    use bevy_shape::{Capsule3d, Sphere, Torus};

    /// A `size` x `size` grid of quads, with its triangles in a scrambled order.
    fn scrambled_grid(size: u32) -> Mesh {
        let row = size + 1;
        let positions: Vec<[f32; 3]> = (0..row * row)
            .map(|i| [(i % row) as f32, (i / row) as f32, 0.0])
            .collect();
        let uvs: Vec<[f32; 2]> = positions.iter().map(|p| [p[0] * 0.5, p[1]]).collect();

        let mut triangles = Vec::new();
        for y in 0..size {
            for x in 0..size {
                let i = y * row + x;
                triangles.push([i, i + 1, i + row]);
                triangles.push([i + 1, i + row + 1, i + row]);
            }
        }
        // Deterministic Fisher-Yates shuffle with a simple LCG.
        let mut state = 0x2545_f491_u32;
        for i in (1..triangles.len()).rev() {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            triangles.swap(i, state as usize % (i + 1));
        }

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(triangles.into_iter().flatten().collect()))
    }

    /// The raw bytes of all the vertex attributes of each corner of each primitive, in a
    /// canonical order. Two meshes with the same soup render the same primitives.
    fn triangle_soup(mesh: &Mesh) -> Vec<[Vec<u8>; 3]> {
        let vertex_bytes = |index: usize| -> Vec<u8> {
            mesh.attributes()
                .flat_map(|(_, values)| values.get_bytes_at(index).to_vec())
                .collect()
        };
        let indices: Vec<usize> = mesh.indices().unwrap().iter().collect();
        let mut soup: Vec<[Vec<u8>; 3]> = indices
            .as_chunks::<3>()
            .0
            .iter()
            .map(|triangle| triangle.map(vertex_bytes))
            .collect();
        soup.sort();
        soup
    }

    fn positions(mesh: &Mesh) -> &[[f32; 3]] {
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("missing positions");
        };
        positions
    }

    #[test]
    fn optimize_vertex_cache_preserves_triangles_and_winding() {
        let mut mesh = scrambled_grid(16);
        let mut triangles_before: Vec<[usize; 3]> = mesh
            .indices()
            .unwrap()
            .iter()
            .collect::<Vec<_>>()
            .as_chunks::<3>()
            .0
            .to_vec();

        mesh.optimize_vertex_cache().unwrap();

        let mut triangles_after: Vec<[usize; 3]> = mesh
            .indices()
            .unwrap()
            .iter()
            .collect::<Vec<_>>()
            .as_chunks::<3>()
            .0
            .to_vec();
        // Triangles are emitted with the exact same corner order, so sorting both lists must
        // give the same result.
        triangles_before.sort_unstable();
        triangles_after.sort_unstable();
        assert_eq!(triangles_before, triangles_after);
    }

    #[test]
    fn optimize_vertex_cache_reduces_transformed_vertices() {
        let mut mesh = scrambled_grid(32);
        let before = mesh.analyze_vertex_cache(16).unwrap();
        mesh.optimize_vertex_cache().unwrap();
        let after = mesh.analyze_vertex_cache(16).unwrap();

        // A scrambled grid has almost no reuse.
        assert!(before.acmr > 2.0, "{before:?}");
        // A regular grid can't go below 0.5 with a finite cache, and good orderings are
        // typically below 0.75 with 16 cached vertices.
        assert!(after.acmr < 0.75, "{after:?}");
        assert_eq!(before.unique_vertices, after.unique_vertices);
    }

    #[test]
    fn optimize_vertex_fetch_orders_vertices_by_first_use() {
        let mut mesh = scrambled_grid(8);
        mesh.optimize_vertex_cache().unwrap();
        let soup_before = triangle_soup(&mesh);

        mesh.optimize_vertex_fetch().unwrap();

        let mut next_new_vertex = 0;
        for index in mesh.indices().unwrap().iter() {
            assert!(
                index <= next_new_vertex,
                "vertex {index} used before {next_new_vertex}"
            );
            if index == next_new_vertex {
                next_new_vertex += 1;
            }
        }
        assert_eq!(next_new_vertex, mesh.count_vertices());
        assert_eq!(soup_before, triangle_soup(&mesh));
    }

    #[test]
    fn optimize_vertex_fetch_reduces_overfetch() {
        let mut mesh = scrambled_grid(64);
        // Scramble the vertices as well, by making the vertex buffer order follow the scrambled
        // triangle order in reverse.
        mesh.optimize_vertex_fetch().unwrap();
        let vertex_count = mesh.count_vertices();
        let reversed: Vec<usize> = (0..vertex_count).rev().collect();
        mesh.try_gather_vertices(|| reversed.iter().copied())
            .unwrap();
        if let Some(Indices::U32(indices)) = mesh.indices_mut() {
            for index in indices {
                *index = (vertex_count - 1) as u32 - *index;
            }
        }
        mesh.optimize_vertex_cache().unwrap();
        let soup_before = triangle_soup(&mesh);
        let before = mesh.analyze_vertex_fetch().unwrap();

        mesh.optimize_vertex_fetch().unwrap();

        let after = mesh.analyze_vertex_fetch().unwrap();
        assert!(
            after.bytes_fetched < before.bytes_fetched,
            "{before:?} {after:?}"
        );
        assert!(after.overfetch < 1.5, "{after:?}");
        assert_eq!(soup_before, triangle_soup(&mesh));
    }

    #[test]
    fn optimize_vertex_fetch_removes_unused_vertices() {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                [0.0, 0.0, 0.0],
                [9.0, 9.0, 9.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
        )
        .with_inserted_indices(Indices::U16(vec![3, 0, 2]));

        mesh.optimize_vertex_fetch().unwrap();

        assert_eq!(mesh.indices(), Some(&Indices::U16(vec![0, 1, 2])));
        assert_eq!(
            positions(&mesh),
            &[[0.0, 1.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]
        );
    }

    #[test]
    fn optimize_for_gpu_keeps_u16_indices() {
        let mut mesh = scrambled_grid(16);
        mesh.compress_indices();
        assert!(matches!(mesh.indices(), Some(Indices::U16(_))));
        mesh.optimize_for_gpu().unwrap();
        assert!(matches!(mesh.indices(), Some(Indices::U16(_))));
    }

    #[test]
    fn optimize_for_gpu_on_primitives() {
        let meshes = [
            Sphere::new(1.0).mesh().uv(32, 18),
            Sphere::new(1.0).mesh().ico(5).unwrap(),
            Torus::default().mesh().build(),
            Capsule3d::default().mesh().build(),
        ];
        for mut mesh in meshes {
            let soup_before = triangle_soup(&mesh);
            let before = mesh.analyze_vertex_cache(32).unwrap();
            let fetch_before = mesh.analyze_vertex_fetch().unwrap();

            mesh.optimize_for_gpu().unwrap();

            let after = mesh.analyze_vertex_cache(32).unwrap();
            let fetch_after = mesh.analyze_vertex_fetch().unwrap();
            assert!(
                after.vertices_transformed <= before.vertices_transformed,
                "{before:?} {after:?}"
            );
            assert!(after.atvr < 1.5, "{after:?}");
            // Reordering triangles for the vertex cache can slightly degrade the vertex fetch
            // locality of meshes that were generated in a perfectly linear order.
            assert!(
                fetch_after.overfetch < fetch_before.overfetch * 1.25,
                "{fetch_before:?} {fetch_after:?}"
            );
            assert_eq!(soup_before, triangle_soup(&mesh));
        }
    }

    #[test]
    fn optimize_handles_degenerate_triangles() {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        )
        .with_inserted_indices(Indices::U32(vec![0, 0, 0, 0, 1, 2, 1, 1, 2, 2, 2, 2]));
        let soup_before = triangle_soup(&mesh);
        mesh.optimize_for_gpu().unwrap();
        assert_eq!(soup_before, triangle_soup(&mesh));
    }

    #[test]
    fn optimize_vertex_fetch_preserves_strip_restarts() {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleStrip,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
            ],
        )
        .with_inserted_indices(Indices::U16(vec![4, 3, 2, u16::MAX, 1, 0, 4]));

        // Vertex cache optimization only supports triangle lists.
        assert!(matches!(
            mesh.optimize_vertex_cache(),
            Err(MeshOptimizationError::UnsupportedTopology(
                PrimitiveTopology::TriangleStrip
            ))
        ));
        mesh.optimize_for_gpu().unwrap();

        assert_eq!(
            mesh.indices(),
            Some(&Indices::U16(vec![0, 1, 2, u16::MAX, 3, 4, 0]))
        );
        assert_eq!(
            positions(&mesh),
            &[
                [4.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 0.0, 0.0]
            ]
        );
    }

    #[test]
    fn optimize_errors() {
        let triangle = || {
            Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            )
            .with_inserted_attribute(
                Mesh::ATTRIBUTE_POSITION,
                vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            )
        };

        assert!(matches!(
            triangle().optimize_for_gpu(),
            Err(MeshOptimizationError::MissingIndices)
        ));
        assert!(matches!(
            triangle()
                .with_inserted_indices(Indices::U32(vec![0, 1]))
                .optimize_for_gpu(),
            Err(MeshOptimizationError::AbruptIndicesEnd(2))
        ));
        assert!(matches!(
            triangle()
                .with_inserted_indices(Indices::U32(vec![0, 1, 3]))
                .optimize_for_gpu(),
            Err(MeshOptimizationError::IndexOutOfBounds {
                index: 3,
                vertex_count: 3
            })
        ));
        assert!(matches!(
            triangle()
                .with_inserted_indices(Indices::U32(vec![0, 1, 2]))
                .analyze_vertex_cache(16),
            Ok(VertexCacheStatistics {
                vertices_transformed: 3,
                unique_vertices: 3,
                ..
            })
        ));
    }

    #[test]
    fn optimize_extracted_mesh_errors() {
        let mut mesh = scrambled_grid(2);
        mesh.take_gpu_data().unwrap();
        assert!(matches!(
            mesh.optimize_for_gpu(),
            Err(MeshOptimizationError::MeshAccessError(_))
        ));
    }

    #[test]
    fn analyze_vertex_cache_without_reuse() {
        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0f32; 3]; 30])
        .with_inserted_indices(Indices::U32((0..30).collect()));
        let statistics = mesh.analyze_vertex_cache(16).unwrap();
        assert_eq!(statistics.vertices_transformed, 30);
        assert_eq!(statistics.acmr, 3.0);
        assert_eq!(statistics.atvr, 1.0);
    }

    #[cfg(feature = "morph")]
    #[test]
    fn optimize_vertex_fetch_remaps_morph_targets() {
        use crate::morph::MorphAttributes;
        use bevy_math::Vec3;

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
        )
        .with_inserted_indices(Indices::U32(vec![2, 0, 1]));
        let morph = |target: f32, vertex: f32| {
            MorphAttributes::new(Vec3::new(target, vertex, 0.0), Vec3::ZERO, Vec3::ZERO)
        };
        // Two targets, stored one after the other.
        mesh.set_morph_targets(vec![
            morph(0.0, 0.0),
            morph(0.0, 1.0),
            morph(0.0, 2.0),
            morph(1.0, 0.0),
            morph(1.0, 1.0),
            morph(1.0, 2.0),
        ]);

        mesh.optimize_vertex_fetch().unwrap();

        assert_eq!(
            positions(&mesh),
            &[[2.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]
        );
        assert_eq!(
            mesh.morph_targets().unwrap(),
            &vec![
                morph(0.0, 2.0),
                morph(0.0, 0.0),
                morph(0.0, 1.0),
                morph(1.0, 2.0),
                morph(1.0, 0.0),
                morph(1.0, 1.0),
            ]
        );
    }

    #[cfg(feature = "morph")]
    #[test]
    fn duplicate_vertices_duplicates_morph_targets() {
        use crate::morph::MorphAttributes;
        use bevy_math::Vec3;

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        )
        .with_inserted_indices(Indices::U32(vec![0, 1, 2, 2, 1, 0]));
        let morph = |x: f32| MorphAttributes::new(Vec3::X * x, Vec3::ZERO, Vec3::ZERO);
        mesh.set_morph_targets(vec![morph(0.0), morph(1.0), morph(2.0)]);

        mesh.duplicate_vertices();

        assert_eq!(mesh.count_vertices(), 6);
        assert_eq!(
            mesh.morph_targets().unwrap(),
            &vec![
                morph(0.0),
                morph(1.0),
                morph(2.0),
                morph(2.0),
                morph(1.0),
                morph(0.0)
            ]
        );
    }
}
