//! Generates levels of detail (LODs) for a mesh with [`Mesh::simplify`], and switches between them
//! depending on the distance to the camera with [`VisibilityRange`].
//!
//! Press `Space` to color the objects according to their level of detail.

use bevy::{camera::visibility::VisibilityRange, mesh::MeshSimplificationSettings, prelude::*};

/// The fraction of the triangles of the original mesh kept by each level of detail, and the
/// distance from the camera at which it stops being used.
const LODS: [(f32, f32); 4] = [(1.0, 12.0), (0.25, 25.0), (0.06, 45.0), (0.015, 200.0)];

/// The size of the crossfade between two levels of detail.
const CROSSFADE: f32 = 2.0;

/// The colors used for each level of detail when coloring objects by level of detail.
const LOD_COLORS: [Color; 4] = [
    Color::srgb(0.2, 0.8, 0.2),
    Color::srgb(0.9, 0.9, 0.2),
    Color::srgb(0.9, 0.5, 0.1),
    Color::srgb(0.9, 0.2, 0.2),
];

/// The materials of the objects, per level of detail.
#[derive(Resource)]
struct LodMaterials {
    normal: Handle<StandardMaterial>,
    colored: [Handle<StandardMaterial>; 4],
    show_colors: bool,
}

/// Marks the mesh entity of a level of detail.
#[derive(Component)]
struct Lod(usize);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (move_camera, toggle_lod_colors))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // A detailed mesh, made cheaper to render by reordering its data for the GPU.
    let mesh = Torus::new(0.6, 1.0)
        .mesh()
        .major_resolution(128)
        .minor_resolution(64)
        .build()
        .optimized_for_gpu()
        .expect("the torus mesh can be optimized");

    // Generate the levels of detail.
    let mut help = String::from("Space: color the objects according to their level of detail\n");
    let mut lods = Vec::new();
    let mut start = 0.0;
    for (level, &(target_ratio, end)) in LODS.iter().enumerate() {
        let mut lod = mesh.clone();
        let error = lod
            .simplify(&MeshSimplificationSettings {
                target_ratio,
                max_error: f32::INFINITY,
                ..default()
            })
            .expect("the torus mesh can be simplified");
        let triangles = lod.indices().map_or(0, |indices| indices.len() / 3);
        help.push_str(&format!(
            "LOD {level}: {triangles} triangles, up to {end} m, error {:.2}%\n",
            error * 100.0
        ));

        // Crossfade between consecutive levels of detail.
        let start_margin = if level == 0 {
            0.0..0.0
        } else {
            start - CROSSFADE..start
        };
        let range = VisibilityRange {
            start_margin,
            end_margin: end - CROSSFADE..end,
            use_aabb: false,
        };
        lods.push((meshes.add(lod), range));
        start = end;
    }

    let normal = materials.add(Color::srgb(0.7, 0.7, 0.75));
    let colored = LOD_COLORS.map(|color| materials.add(color));
    commands.insert_resource(LodMaterials {
        normal: normal.clone(),
        colored,
        show_colors: false,
    });

    // A field of objects, each with all its levels of detail.
    for x in -6..=6 {
        for z in 0..24 {
            commands
                .spawn((
                    Transform::from_xyz(x as f32 * 4.0, 1.0, z as f32 * -6.0)
                        .with_rotation(Quat::from_rotation_x((x + z) as f32 * 0.4)),
                    Visibility::default(),
                ))
                .with_children(|object| {
                    for (level, (mesh, range)) in lods.iter().enumerate() {
                        object.spawn((
                            Mesh3d(mesh.clone()),
                            MeshMaterial3d(normal.clone()),
                            range.clone(),
                            Lod(level),
                        ));
                    }
                });
        }
    }

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(200.0, 400.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.3, 0.2))),
        Transform::from_xyz(0.0, -1.0, -100.0),
    ));
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 6.0, 12.0).looking_at(Vec3::new(0.0, 0.0, -10.0), Vec3::Y),
    ));

    commands.spawn((
        Text::new(help),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));
}

/// Moves the camera back and forth over the field of objects.
fn move_camera(time: Res<Time>, mut cameras: Query<&mut Transform, With<Camera3d>>) {
    for mut transform in &mut cameras {
        let z = 12.0 - 60.0 * (0.5 - 0.5 * ops::cos(time.elapsed_secs() * 0.2));
        *transform =
            Transform::from_xyz(0.0, 6.0, z).looking_at(Vec3::new(0.0, 0.0, z - 22.0), Vec3::Y);
    }
}

/// Toggles the coloring of the objects according to their level of detail.
fn toggle_lod_colors(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut lod_materials: ResMut<LodMaterials>,
    mut lods: Query<(&Lod, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }
    lod_materials.show_colors = !lod_materials.show_colors;
    for (lod, mut material) in &mut lods {
        material.0 = if lod_materials.show_colors {
            lod_materials.colored[lod.0].clone()
        } else {
            lod_materials.normal.clone()
        };
    }
}
