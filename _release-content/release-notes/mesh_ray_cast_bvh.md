---
title: Faster mesh picking with cached BVHs
authors: ["@luidgi-gaudin"]
pull_requests: []
---

Mesh picking and `MeshRayCast` used to test every triangle of the meshes whose bounding box was
hit by the ray. This is fine for simple meshes, but on large meshes like terrains or detailed
props, each ray could take several milliseconds, every frame, for every pointer.

Ray casts now use a bounding volume hierarchy (BVH) for meshes with more than a few triangles.
The BVH sorts the triangles of the mesh into a tree of nested boxes, so that only the few
triangles close to the ray need to be tested, nearest first:

| Triangles | Without BVH | With BVH |
|-----------|-------------|----------|
| 20,000    | 105 µs      | 0.34 µs  |
| 2,000,000 | 13.8 ms     | 0.39 µs  |

This is completely automatic when using the `MeshPickingPlugin`, and gives exactly the same results
as before:

- The BVH of a mesh is built once rays have hit it over a few frames without it being modified,
  and cached in the new `MeshRayCastCache` resource. Meshes that are modified often are never
  slowed down by rebuilding their BVH. The BVHs of very large meshes are built in a background
  task, so that building them doesn't cause long frame hitches.
- The BVH is dropped when the mesh is modified or removed. Ray casts made after modifying a mesh in
  the same frame don't use its cached BVH, so they never see stale data, while the BVHs of the other
  meshes keep being used.

If you use `MeshRayCast` without the `MeshPickingPlugin`, add the `MeshRayCastPlugin` to enable the
cache. To ray cast against your own mesh data, you can build a `TriangleBvh` yourself and pass it to
`ray_mesh_intersection_with_bvh`.
