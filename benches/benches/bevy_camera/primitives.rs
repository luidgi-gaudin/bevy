use bevy_camera::{
    primitives::{Aabb, Frustum, Sphere},
    CameraProjection, PerspectiveProjection,
};
use bevy_math::{Affine3A, EulerRot, Quat, Vec3, Vec3A, Vec4};
use bevy_shape::{HalfSpace, ViewFrustum};
use bevy_transform::components::GlobalTransform;
use chacha20::ChaCha8Rng;
use core::{f32::consts::PI, hint::black_box};
use criterion::{criterion_group, BenchmarkId, Criterion, Throughput};
use rand::{RngExt, SeedableRng};

pub fn intersects_obb(c: &mut Criterion) {
    let mut group = c.benchmark_group("intersects_obb");

    let aabb = Aabb {
        center: Vec3A::ZERO,
        half_extents: Vec3A::new(0.5, 0.5, 0.5),
    };

    let world_from_local = Affine3A::from_rotation_translation(
        Quat::from_rotation_y(std::f32::consts::FRAC_PI_4),
        Vec3::new(1.0, 0.5, -0.5),
    );

    let identity_transform = Affine3A::IDENTITY;

    let sphere = Sphere {
        center: Vec3A::new(1.0, 0.5, 0.0),
        radius: 1.5,
    };

    let frustum = Frustum(ViewFrustum {
        half_spaces: [
            HalfSpace::new(Vec4::new(-0.9701, -0.2425, -0.0000, 0.7276)),
            HalfSpace::new(Vec4::new(-0.0000, 1.0000, -0.0000, 1.0000)),
            HalfSpace::new(Vec4::new(-0.0000, -0.2425, -0.9701, 0.7276)),
            HalfSpace::new(Vec4::new(-0.0000, -1.0000, -0.0000, 1.0000)),
            HalfSpace::new(Vec4::new(-0.0000, -0.2425, 0.9701, 0.7276)),
            HalfSpace::new(Vec4::new(0.9701, -0.2425, -0.0000, 0.7276)),
        ],
    });

    assert!(sphere.intersects_obb(&aabb, &world_from_local));
    group.bench_function("sphere_intersects_obb", |b| {
        b.iter(|| black_box(sphere.intersects_obb(black_box(&aabb), black_box(&world_from_local))));
    });

    assert!(frustum.intersects_obb(&aabb, &world_from_local, true, true));
    group.bench_function("frustum_intersects_obb", |b| {
        b.iter(|| {
            black_box(frustum.intersects_obb(
                black_box(&aabb),
                black_box(&world_from_local),
                black_box(true), // intersect_near
                black_box(true), // intersect_far
            ))
        });
    });

    assert!(frustum.intersects_obb(&aabb, &identity_transform, true, true));
    group.bench_function("frustum_intersects_obb_fallback_identity", |b| {
        b.iter(|| {
            black_box(frustum.intersects_obb(
                black_box(&aabb),
                black_box(&identity_transform),
                black_box(true),
                black_box(true),
            ))
        });
    });

    assert!(frustum.intersects_obb_identity(&aabb));
    group.bench_function("frustum_intersects_obb_identity", |b| {
        b.iter(|| black_box(frustum.intersects_obb_identity(black_box(&aabb))));
    });

    group.finish();
}

/// Culls 10,000 randomly placed, rotated and scaled boxes against a perspective camera frustum,
/// like `check_visibility_cpu_culling` does for every entity, every frame.
pub fn frustum_culling(c: &mut Criterion) {
    const ENTITIES: usize = 10_000;
    let mut group = c.benchmark_group("frustum_culling");
    group.throughput(Throughput::Elements(ENTITIES as u64));

    // A camera at the origin, looking towards -Z, with a 70 degree vertical field of view.
    let projection = PerspectiveProjection {
        fov: 70.0_f32.to_radians(),
        aspect_ratio: 16.0 / 9.0,
        ..Default::default()
    };
    let frustum = Frustum(ViewFrustum::from_clip_from_world(
        &projection.get_clip_from_view(),
    ));

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut random_entities = |visible_only: bool| -> Vec<(Aabb, GlobalTransform)> {
        let mut entities = Vec::with_capacity(ENTITIES);
        while entities.len() < ENTITIES {
            let aabb = Aabb {
                center: Vec3A::ZERO,
                half_extents: Vec3A::new(
                    rng.random_range(0.1..2.0),
                    rng.random_range(0.1..2.0),
                    rng.random_range(0.1..2.0),
                ),
            };
            let transform = GlobalTransform::from(Affine3A::from_scale_rotation_translation(
                Vec3::splat(rng.random_range(0.5..2.0)),
                Quat::from_euler(
                    EulerRot::YXZ,
                    rng.random_range(-PI..PI),
                    rng.random_range(-PI..PI),
                    rng.random_range(-PI..PI),
                ),
                Vec3::new(
                    rng.random_range(-100.0..100.0),
                    rng.random_range(-100.0..100.0),
                    rng.random_range(-100.0..100.0),
                ),
            ));
            if !visible_only || cull_sphere_then_obb(&frustum, &aabb, &transform) {
                entities.push((aabb, transform));
            }
        }
        entities
    };

    for (name, entities) in [
        ("mixed", random_entities(false)),
        ("visible", random_entities(true)),
    ] {
        let visible = |cull: fn(&Frustum, &Aabb, &GlobalTransform) -> bool| {
            entities
                .iter()
                .filter(|(aabb, transform)| cull(black_box(&frustum), aabb, transform))
                .count()
        };
        // Cameras test a bounding sphere, then the box.
        assert_eq!(
            visible(cull_sphere_then_obb),
            visible(cull_obb_with_bounding_sphere)
        );
        group.bench_function(BenchmarkId::new("sphere_then_obb", name), |b| {
            b.iter(|| visible(cull_sphere_then_obb));
        });
        group.bench_function(BenchmarkId::new("obb_with_bounding_sphere", name), |b| {
            b.iter(|| visible(cull_obb_with_bounding_sphere));
        });

        // Shadow-casting lights only test the box.
        assert_eq!(visible(cull_obb), visible(cull_obb_with_containing_sphere));
        group.bench_function(BenchmarkId::new("obb", name), |b| {
            b.iter(|| visible(cull_obb));
        });
        group.bench_function(BenchmarkId::new("obb_with_containing_sphere", name), |b| {
            b.iter(|| visible(cull_obb_with_containing_sphere));
        });
    }

    group.finish();
}

/// Tests the bounding sphere of the box against the frustum, then the box itself.
fn cull_sphere_then_obb(frustum: &Frustum, aabb: &Aabb, transform: &GlobalTransform) -> bool {
    let world_from_local = transform.affine();
    let sphere = Sphere {
        center: world_from_local.transform_point3a(aabb.center),
        radius: transform.radius_vec3a(aabb.half_extents),
    };
    frustum.intersects_sphere(&sphere, false)
        && frustum.intersects_obb(aabb, &world_from_local, true, false)
}

/// Tests the box against the frustum, skipping the planes its bounding sphere is inside of.
fn cull_obb_with_bounding_sphere(
    frustum: &Frustum,
    aabb: &Aabb,
    transform: &GlobalTransform,
) -> bool {
    frustum.intersects_obb_with_bounding_sphere(
        aabb,
        &transform.affine(),
        transform.radius_vec3a(aabb.half_extents),
        true,
        false,
    )
}

/// Tests the box against the frustum.
fn cull_obb(frustum: &Frustum, aabb: &Aabb, transform: &GlobalTransform) -> bool {
    frustum.intersects_obb(aabb, &transform.affine(), true, false)
}

/// Tests the box against the frustum, with a sphere that contains it to skip most planes.
fn cull_obb_with_containing_sphere(
    frustum: &Frustum,
    aabb: &Aabb,
    transform: &GlobalTransform,
) -> bool {
    let world_from_local = transform.affine();
    frustum.intersects_obb_with_bounding_sphere(
        aabb,
        &world_from_local,
        aabb.bounding_sphere_radius(&world_from_local.matrix3),
        true,
        false,
    )
}

criterion_group!(benches, intersects_obb, frustum_culling);
