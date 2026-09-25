---
title: Faster frustum culling
authors: ["@luidgi-gaudin"]
pull_requests: []
---

Every frame, Bevy checks which entities are inside the frustum of each camera and of each
shadow-casting light, so that it doesn't render the others. For each entity, cameras tested a
bounding sphere against the planes of the frustum, then the oriented bounding box of the entity
against all of these planes again.

The oriented bounding box is now only tested against the planes its center is outside of: an
object whose center is inside of a plane can't be entirely outside of it. For an entity whose
center is inside the frustum, the common case for visible entities, each plane now costs two dot
products instead of six. Shadow-casting lights, which only tested the oriented bounding box, now
use the same approach with a bounding sphere that contains the whole box.

The results are exactly the same as before. Culling 10,000 entities against a camera frustum:

| Entities                 | Before | After  |
|--------------------------|--------|--------|
| Camera, all visible      | 223 µs | 100 µs |
| Camera, mixed scene      | 128 µs | 125 µs |
| Shadows, all visible     | 137 µs | 105 µs |
| Shadows, mixed scene     | 155 µs | 144 µs |

This is automatic. If you do your own culling, use the new
`Frustum::intersects_obb_with_bounding_sphere`, and `Aabb::bounding_sphere_radius` for a sphere
that contains the whole box.
