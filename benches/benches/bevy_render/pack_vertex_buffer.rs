use core::hint::black_box;

use benches::bench;
use criterion::{criterion_group, Criterion, Throughput};
use wgpu_types::WriteOnly;

use bevy_asset::RenderAssetUsages;
use bevy_mesh::{Mesh, PrimitiveTopology};

const VERTEX_COUNT: usize = 1 << 16;

/// A mesh with the vertex attributes that are typically used with `StandardMaterial`.
fn mesh() -> Mesh {
    let positions: Vec<[f32; 3]> = (0..VERTEX_COUNT)
        .map(|i| [i as f32, (i % 7) as f32, (i % 13) as f32])
        .collect();
    let normals = vec![[0.0, 1.0, 0.0]; VERTEX_COUNT];
    let uvs: Vec<[f32; 2]> = (0..VERTEX_COUNT)
        .map(|i| [(i % 256) as f32 / 256.0, (i / 256) as f32 / 256.0])
        .collect();
    let tangents = vec![[1.0, 0.0, 0.0, 1.0]; VERTEX_COUNT];

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_TANGENT, tangents)
}

fn pack_vertex_buffer(c: &mut Criterion) {
    let mut group = c.benchmark_group(bench!("pack_vertex_buffer"));
    group.throughput(Throughput::Elements(VERTEX_COUNT as u64));

    let mesh = mesh();
    let mut buffer = vec![0u8; mesh.get_vertex_buffer_size()];
    group.bench_function("position_normal_uv_tangent", |b| {
        b.iter(|| {
            mesh.write_packed_vertex_buffer_data(WriteOnly::from_mut(black_box(&mut buffer[..])));
        });
    });

    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[1.0f32; 3]; VERTEX_COUNT]);
    let mut buffer = vec![0u8; mesh.get_vertex_buffer_size()];
    group.bench_function("position", |b| {
        b.iter(|| {
            mesh.write_packed_vertex_buffer_data(WriteOnly::from_mut(black_box(&mut buffer[..])));
        });
    });

    group.finish();
}

criterion_group!(benches, pack_vertex_buffer);
