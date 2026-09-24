---
title: Mesh simplification for levels of detail
authors: ["@luidgi-gaudin"]
pull_requests: []
---

Rendering a detailed mesh that only covers a few pixels on screen wastes a lot of GPU time. The
usual fix is to use simplified versions of the mesh, called levels of detail (LODs), for objects
that are far away from the camera. Bevy has supported switching between LODs with
`VisibilityRange` for a while, but creating the simplified meshes was up to you.

`Mesh::simplify` now generates them for you:

```rust
let lod = mesh.clone().simplified(&MeshSimplificationSettings {
    // Keep 25% of the triangles...
    target_ratio: 0.25,
    // ...unless that would move the surface by more than 1% of the size of the mesh.
    max_error: 0.01,
    ..default()
})?;
```

The simplification repeatedly collapses the edges whose removal changes the shape of the mesh the
least, measured with quadric error metrics. It is careful to keep meshes looking right:

- Vertices are only merged into other existing vertices, so normals, UVs, colors and joint
  weights stay valid, and no new vertex is created.
- Borders and attribute seams (like the edges of UV islands, or hard edges with split normals)
  keep their shape, and attributes never bleed across seams.
- Collapses that would flip triangles are rejected.
- `lock_border` keeps the borders of the mesh completely in place, so that terrain chunks
  simplified separately still fit together.

The error of the simplified mesh is returned, so you can decide at which distance to switch to it.
Combined with `VisibilityRange`, this gives you a full LOD pipeline:

```rust
let lods = [(1.0, 0.0..20.0), (0.25, 20.0..60.0), (0.05, 60.0..200.0)];
for (target_ratio, range) in lods {
    let mesh = mesh.clone().simplified(&MeshSimplificationSettings {
        target_ratio,
        max_error: f32::INFINITY,
        ..default()
    })?;
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material.clone()),
        VisibilityRange::abrupt(range.start, range.end),
    ));
}
```

Simplified meshes are also optimized with `Mesh::optimize_for_gpu`, so they are as cheap to
render as possible.
