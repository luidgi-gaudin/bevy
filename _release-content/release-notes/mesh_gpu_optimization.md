---
title: Mesh optimization for the GPU
authors: ["@luidgi-gaudin"]
pull_requests: []
---

The order of the triangles and vertices of a mesh doesn't change what it looks like, but it has a
big impact on how fast it renders. GPUs keep the output of the vertex shader for the last few
vertices they processed in a small "post-transform" cache, and fetch vertex attributes from memory
in cache lines. When the triangles that share a vertex are drawn far apart from each other, that
vertex is shaded several times, and when vertices that are used together are stored far apart,
the GPU wastes memory bandwidth.

Many meshes, whether they come from authoring tools or are procedurally generated, are not stored
in a GPU-friendly order. Bevy can now fix that for you:

```rust
// Reorder triangles to maximize vertex cache reuse, then reorder vertices (removing unused ones)
// so that they are fetched linearly.
mesh.optimize_for_gpu()?;

// Or apply each step separately:
mesh.optimize_vertex_cache()?;
mesh.optimize_vertex_fetch()?;
```

The triangle reordering implements Tom Forsyth's
[Linear-Speed Vertex Cache Optimisation](https://tomforsyth1000.github.io/papers/fast_vert_cache_opt.html),
the same family of algorithms used by [meshoptimizer](https://github.com/zeux/meshoptimizer). It is
written in pure Rust, so it works on every platform Bevy supports, including the web. Indices keep
their format, triangle winding is preserved, and morph targets are reordered along with the other
vertex attributes.

You can also measure how well a mesh uses the GPU caches, for example to decide whether it is worth
optimizing a mesh at runtime:

```rust
let stats = mesh.analyze_vertex_cache(16)?;
info!("{} vertex shader invocations per triangle", stats.acmr);

let fetch = mesh.analyze_vertex_fetch()?;
info!("{}x more bytes fetched than needed", fetch.overfetch);
```

On Bevy's built-in sphere, torus and capsule meshes, this reduces the number of vertex shader
invocations by about 35%. On meshes with a poor triangle order, the reduction can reach 75%, and
the vertex memory traffic can be divided by more than 10.

glTF meshes can be optimized as they are loaded, either for all glTF files or for specific ones:

```rust
app.add_plugins(DefaultPlugins.set(GltfPlugin {
    optimize_meshes: true,
    ..default()
}));

// Or per asset:
let scene = asset_server
    .load_builder()
    .with_settings(|settings: &mut GltfLoaderSettings| {
        settings.optimize_meshes = Some(true);
    })
    .load("models/my_model.glb#Scene0");
```

This is disabled by default, as it makes loading slightly slower and changes the order of the
vertices of the loaded meshes. There is no need to enable it for files that have already been
optimized by a tool like `gltfpack`.
