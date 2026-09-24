use core::hint::black_box;

use benches::bench;
use chacha20::ChaCha8Rng;
use criterion::{criterion_group, BatchSize, Criterion};
use rand::{prelude::SliceRandom, SeedableRng};

use bevy_asset::RenderAssetUsages;
use bevy_mesh::{Indices, Mesh, MeshSimplificationSettings, Meshable, PrimitiveTopology};
use bevy_shape::Sphere;

const GRID_SIZE: u32 = 256;

/// A grid of `GRID_SIZE` x `GRID_SIZE` quads, with its triangles in a random order, as is common
/// for meshes exported without optimization.
fn scrambled_grid() -> Mesh {
    let row = GRID_SIZE + 1;
    let positions: Vec<[f32; 3]> = (0..row * row)
        .map(|i| [(i % row) as f32, (i / row) as f32, 0.0])
        .collect();
    let normals = vec![[0.0, 0.0, 1.0]; positions.len()];
    let uvs: Vec<[f32; 2]> = positions.iter().map(|p| [p[0], p[1]]).collect();

    let mut triangles: Vec<[u32; 3]> = (0..GRID_SIZE)
        .flat_map(|y| (0..GRID_SIZE).map(move |x| y * row + x))
        .flat_map(|i| [[i, i + 1, i + row], [i + 1, i + row + 1, i + row]])
        .collect();
    triangles.shuffle(&mut ChaCha8Rng::seed_from_u64(42));

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(triangles.into_iter().flatten().collect()))
}

fn optimize_mesh(c: &mut Criterion) {
    let mesh = scrambled_grid();
    let mut group = c.benchmark_group(bench!("scrambled_grid"));

    group.bench_function("vertex_cache", |b| {
        b.iter_batched_ref(
            || mesh.clone(),
            |mesh| mesh.optimize_vertex_cache().unwrap(),
            BatchSize::LargeInput,
        );
    });

    group.bench_function("vertex_fetch", |b| {
        b.iter_batched_ref(
            || mesh.clone(),
            |mesh| mesh.optimize_vertex_fetch().unwrap(),
            BatchSize::LargeInput,
        );
    });

    group.bench_function("for_gpu", |b| {
        b.iter_batched_ref(
            || mesh.clone(),
            |mesh| mesh.optimize_for_gpu().unwrap(),
            BatchSize::LargeInput,
        );
    });

    group.bench_function("analyze_vertex_cache", |b| {
        b.iter(|| black_box(mesh.analyze_vertex_cache(black_box(16)).unwrap()));
    });

    group.finish();

    let mut group = c.benchmark_group(bench!("simplify"));
    let sphere = Sphere::new(1.0).mesh().uv(256, 128);
    for target_ratio in [0.5, 0.1] {
        group.bench_function(format!("uv_sphere_{target_ratio}"), |b| {
            let settings = MeshSimplificationSettings {
                target_ratio,
                max_error: f32::INFINITY,
                lock_border: false,
            };
            b.iter_batched_ref(
                || sphere.clone(),
                |mesh| mesh.simplify(&settings).unwrap(),
                BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

criterion_group!(benches, optimize_mesh);
