//! Ray casting for meshes.
//!
//! See the [`MeshRayCast`] system parameter for more information.

mod bvh;
mod intersections;

use alloc::sync::Arc;
use bevy_app::{App, Plugin, PostUpdate};
use bevy_derive::{Deref, DerefMut};
use bevy_tasks::AsyncComputeTaskPool;
use core::sync::atomic::{AtomicU64, Ordering};

use bevy_camera::{
    primitives::Aabb,
    visibility::{InheritedVisibility, ViewVisibility},
};
use bevy_mesh::{Mesh, Mesh2d, Mesh3d};
use bevy_reflect::{std_traits::ReflectDefault, Reflect};
use bevy_shape::{Aabb3d, Ray3d};

pub use bvh::TriangleBvh;
use intersections::*;
pub use intersections::{
    ray_aabb_intersection_3d, ray_mesh_intersection, ray_mesh_intersection_with_bvh, RayMeshHit,
};

use bevy_asset::{AssetEvent, AssetEventSystems, AssetId, Assets, Handle};
use bevy_ecs::{
    change_detection::Tick, prelude::*, system::lifetimeless::Read, system::SystemParam,
};
use bevy_math::FloatOrd;
use bevy_platform::{collections::HashMap, sync::RwLock};
use bevy_transform::components::GlobalTransform;
use tracing::*;

/// How a ray cast should handle [`Visibility`](bevy_camera::visibility::Visibility).
#[derive(Clone, Copy, Reflect)]
#[reflect(Clone)]
pub enum RayCastVisibility {
    /// Completely ignore visibility checks. Hidden items can still be ray cast against.
    Any,
    /// Only cast rays against entities that are visible in the hierarchy. See [`Visibility`](bevy_camera::visibility::Visibility).
    Visible,
    /// Only cast rays against entities that are visible in the hierarchy and visible to a camera or
    /// light. See [`Visibility`](bevy_camera::visibility::Visibility).
    VisibleInView,
}

/// Settings for a ray cast.
#[derive(Clone)]
pub struct MeshRayCastSettings<'a> {
    /// Determines how ray casting should consider [`Visibility`](bevy_camera::visibility::Visibility).
    pub visibility: RayCastVisibility,
    /// A predicate that is applied for every entity that ray casts are performed against.
    /// Only entities that return `true` will be considered.
    pub filter: &'a dyn Fn(Entity) -> bool,
    /// A function that is run every time a hit is found. Ray casting will continue to check for hits
    /// along the ray as long as this returns `false`.
    pub early_exit_test: &'a dyn Fn(Entity) -> bool,
}

impl<'a> MeshRayCastSettings<'a> {
    /// Set the filter to apply to the ray cast.
    pub fn with_filter(mut self, filter: &'a impl Fn(Entity) -> bool) -> Self {
        self.filter = filter;
        self
    }

    /// Set the early exit test to apply to the ray cast.
    pub fn with_early_exit_test(mut self, early_exit_test: &'a impl Fn(Entity) -> bool) -> Self {
        self.early_exit_test = early_exit_test;
        self
    }

    /// Set the [`RayCastVisibility`] setting to apply to the ray cast.
    pub fn with_visibility(mut self, visibility: RayCastVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// This ray cast should exit as soon as the nearest hit is found.
    pub fn always_early_exit(self) -> Self {
        self.with_early_exit_test(&|_| true)
    }

    /// This ray cast should check all entities whose AABB intersects the ray and return all hits.
    pub fn never_early_exit(self) -> Self {
        self.with_early_exit_test(&|_| false)
    }
}

impl<'a> Default for MeshRayCastSettings<'a> {
    fn default() -> Self {
        Self {
            visibility: RayCastVisibility::VisibleInView,
            filter: &|_| true,
            early_exit_test: &|_| true,
        }
    }
}

/// Determines whether backfaces should be culled or included in ray intersection tests.
///
/// By default, backfaces are culled.
#[derive(Copy, Clone, Default, Reflect)]
#[reflect(Default, Clone)]
pub enum Backfaces {
    /// Cull backfaces.
    #[default]
    Cull,
    /// Include backfaces.
    Include,
}

/// Disables backface culling for [ray casts](MeshRayCast) on this entity.
#[derive(Component, Copy, Clone, Default, Reflect)]
#[reflect(Component, Default, Clone)]
pub struct RayCastBackfaces;

/// A simplified mesh component that can be used for [ray casting](super::MeshRayCast).
///
/// Consider using this component for complex meshes that don't need perfectly accurate ray casting.
#[derive(Component, FromTemplate, Clone, Debug, Deref, DerefMut, Reflect)]
#[reflect(Component, Debug, Clone)]
pub struct SimplifiedMesh(pub Handle<Mesh>);

/// Meshes with fewer triangles than this are always ray cast by testing all their triangles, as
/// this is fast enough.
const MIN_BVH_TRIANGLES: usize = 128;

/// The BVHs of meshes with at least this many triangles are built in a background task, instead
/// of blocking the ray cast that needs it for several milliseconds.
const MIN_ASYNC_BVH_TRIANGLES: usize = 1 << 16;

/// Adds the [`MeshRayCastCache`], which speeds up [`MeshRayCast`] ray casts against large meshes.
///
/// This is added by the [`MeshPickingPlugin`](super::MeshPickingPlugin).
#[derive(Default)]
pub struct MeshRayCastPlugin;

impl Plugin for MeshRayCastPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeshRayCastCache>().add_systems(
            PostUpdate,
            sync_mesh_ray_cast_cache.after(AssetEventSystems),
        );
    }
}

/// Caches a [bounding volume hierarchy](TriangleBvh) for large meshes that are hit by
/// [`MeshRayCast`] ray casts, so that only a few of their triangles need to be tested.
///
/// The BVH of a mesh is built the second time a ray is cast against it, and is dropped when the
/// mesh is modified or removed. This makes the ray casts against large static meshes orders of
/// magnitude faster, without slowing down ray casts against meshes that are modified every frame.
/// The BVHs of very large meshes are built in a background task: until it completes, ray casts
/// test all the triangles of these meshes.
///
/// Ray casts give exactly the same results with and without this cache. Cached BVHs are only used
/// if the [`AssetEvent`]s of all the changes to the meshes have been processed: after a mesh is
/// modified, ray casts go through all the triangles of the meshes until the end of the frame.
/// Changes made to a mesh without sending an [`AssetEvent::Modified`] event (like with
/// [`Assets::get_mut_untracked`]) are not detected.
///
/// Add the [`MeshRayCastPlugin`] to use this cache.
#[derive(Resource, Default)]
pub struct MeshRayCastCache {
    /// This is shared with the tasks that build BVHs in the background.
    entries: Arc<RwLock<HashMap<AssetId<Mesh>, CacheEntry>>>,
    /// The change tick of [`Assets<Mesh>`] once all the [`AssetEvent`]s of the changes to the
    /// meshes have been processed, or `None` if some of them haven't been sent yet.
    ///
    /// If `Assets<Mesh>` changed since then, a mesh might have been modified without the cache
    /// knowing about it yet, so the cached BVHs can't be used.
    synchronized_tick: Option<Tick>,
    /// Identifies the background tasks that build BVHs.
    next_task_id: AtomicU64,
}

enum CacheEntry {
    /// A ray was cast once against the mesh since it was added or last modified.
    CastOnce,
    /// The BVH of the mesh is being built by the background task with this id.
    Building(u64),
    /// The BVH of the mesh.
    Built(Arc<TriangleBvh>),
}

impl MeshRayCastCache {
    /// Returns the BVH of a mesh, building it if the mesh has been ray cast before.
    ///
    /// Returns `None` if the mesh has few triangles, if it is its first ray cast since it was
    /// modified, if its BVH is being built, or if meshes might have been modified since the cache
    /// was last synchronized.
    fn bvh(
        &self,
        id: AssetId<Mesh>,
        mesh: &Mesh,
        meshes_changed_tick: Tick,
    ) -> Option<Arc<TriangleBvh>> {
        if self.synchronized_tick != Some(meshes_changed_tick) {
            return None;
        }
        let (positions, indices) = mesh_triangles(mesh)?;
        let triangle_count = triangle_count(positions, indices);
        if triangle_count < MIN_BVH_TRIANGLES {
            return None;
        }

        // Release the read lock before taking the write lock.
        let entry = self
            .entries
            .read()
            .unwrap()
            .get(&id)
            .map(|entry| match entry {
                CacheEntry::CastOnce => Ok(()),
                CacheEntry::Building(_) => Err(None),
                CacheEntry::Built(bvh) => Err(Some(bvh.clone())),
            });
        match entry {
            Some(Err(bvh)) => return bvh,
            Some(Ok(())) => {}
            None => {
                self.entries
                    .write()
                    .unwrap()
                    .insert(id, CacheEntry::CastOnce);
                return None;
            }
        }

        if triangle_count >= MIN_ASYNC_BVH_TRIANGLES
            && let Some(task_pool) = AsyncComputeTaskPool::try_get()
        {
            let task_id = self.next_task_id.fetch_add(1, Ordering::Relaxed);
            self.entries
                .write()
                .unwrap()
                .insert(id, CacheEntry::Building(task_id));
            // The mesh can be modified while the BVH is being built, so the task needs its own
            // copy of the data. If the mesh is modified, the cache entry is removed, so the BVH
            // built from the old data will not be used.
            let positions = positions.to_vec();
            let indices = indices.cloned();
            let entries = self.entries.clone();
            task_pool
                .spawn(async move {
                    let bvh = {
                        let _span = debug_span!("build_mesh_bvh").entered();
                        build_bvh(&positions, indices.as_ref())
                    };
                    if let Some(entry) = entries.write().unwrap().get_mut(&id)
                        && matches!(entry, CacheEntry::Building(id) if *id == task_id)
                    {
                        *entry = CacheEntry::Built(Arc::new(bvh));
                    }
                })
                .detach();
            return None;
        }

        let bvh = {
            let _span = debug_span!("build_mesh_bvh").entered();
            Arc::new(build_bvh(positions, indices))
        };
        self.entries
            .write()
            .unwrap()
            .insert(id, CacheEntry::Built(bvh.clone()));
        Some(bvh)
    }

    /// Removes all the cached BVHs.
    pub fn clear(&mut self) {
        self.entries.write().unwrap().clear();
    }
}

/// Drops the cached BVHs of the meshes that were modified or removed.
pub fn sync_mesh_ray_cast_cache(
    mut cache: ResMut<MeshRayCastCache>,
    mut mesh_events: MessageReader<AssetEvent<Mesh>>,
    meshes: Res<Assets<Mesh>>,
) {
    let cache = cache.bypass_change_detection();
    let mut entries = cache.entries.write().unwrap();
    for event in mesh_events.read() {
        match event {
            AssetEvent::Added { id }
            | AssetEvent::Modified { id }
            | AssetEvent::Removed { id }
            | AssetEvent::Unused { id } => {
                entries.remove(id);
            }
            AssetEvent::LoadedWithDependencies { .. } => {}
        }
    }
    drop(entries);
    cache.synchronized_tick = (!meshes.has_pending_events()).then(|| meshes.last_changed());
}

type MeshFilter = Or<(With<Mesh3d>, With<Mesh2d>, With<SimplifiedMesh>)>;

/// Add this ray casting [`SystemParam`] to your system to cast rays into the world with an
/// immediate-mode API. Call `cast_ray` to immediately perform a ray cast and get a result.
///
/// Under the hood, this is a collection of regular bevy queries, resources, and local parameters
/// that are added to your system.
///
/// ## Usage
///
/// The following system casts a ray into the world with the ray positioned at the origin, pointing in
/// the X-direction, and returns a list of intersections:
///
/// ```
/// # use bevy_math::prelude::*;
/// # use bevy_shape::prelude::*;
/// # use bevy_picking::prelude::*;
/// fn ray_cast_system(mut ray_cast: MeshRayCast) {
///     let ray = Ray3d::new(Vec3::ZERO, Dir3::X);
///     let hits = ray_cast.cast_ray(ray, &MeshRayCastSettings::default());
/// }
/// ```
///
/// ## Configuration
///
/// You can specify the behavior of the ray cast using [`MeshRayCastSettings`]. This allows you to filter out
/// entities, configure early-out behavior, and set whether the [`Visibility`](bevy_camera::visibility::Visibility)
/// of an entity should be considered.
///
/// ```
/// # use bevy_ecs::prelude::*;
/// # use bevy_math::prelude::*;
/// # use bevy_shape::prelude::*;
/// # use bevy_picking::prelude::*;
/// # #[derive(Component)]
/// # struct Foo;
/// fn ray_cast_system(mut ray_cast: MeshRayCast, foo_query: Query<(), With<Foo>>) {
///     let ray = Ray3d::new(Vec3::ZERO, Dir3::X);
///
///     // Only ray cast against entities with the `Foo` component.
///     let filter = |entity| foo_query.contains(entity);
///
///     // Never early-exit. Note that you can change behavior per-entity.
///     let early_exit_test = |_entity| false;
///
///     // Ignore the visibility of entities. This allows ray casting hidden entities.
///     let visibility = RayCastVisibility::Any;
///
///     let settings = MeshRayCastSettings::default()
///         .with_filter(&filter)
///         .with_early_exit_test(&early_exit_test)
///         .with_visibility(visibility);
///
///     // Cast the ray with the settings, returning a list of intersections.
///     let hits = ray_cast.cast_ray(ray, &settings);
/// }
/// ```
#[derive(SystemParam)]
pub struct MeshRayCast<'w, 's> {
    #[doc(hidden)]
    pub meshes: Res<'w, Assets<Mesh>>,
    #[doc(hidden)]
    pub cache: Option<Res<'w, MeshRayCastCache>>,
    #[doc(hidden)]
    pub hits: Local<'s, Vec<(FloatOrd, (Entity, RayMeshHit))>>,
    #[doc(hidden)]
    pub output: Local<'s, Vec<(Entity, RayMeshHit)>>,
    #[doc(hidden)]
    pub culled_list: Local<'s, Vec<(FloatOrd, Entity)>>,
    #[doc(hidden)]
    pub culling_query: Query<
        'w,
        's,
        (
            Read<InheritedVisibility>,
            Read<ViewVisibility>,
            Read<Aabb>,
            Read<GlobalTransform>,
            Entity,
        ),
        MeshFilter,
    >,
    #[doc(hidden)]
    pub mesh_query: Query<
        'w,
        's,
        (
            Option<Read<Mesh2d>>,
            Option<Read<Mesh3d>>,
            Option<Read<SimplifiedMesh>>,
            Has<RayCastBackfaces>,
            Read<GlobalTransform>,
        ),
        MeshFilter,
    >,
}

impl<'w, 's> MeshRayCast<'w, 's> {
    /// Casts the `ray` into the world and returns a sorted list of intersections, nearest first.
    pub fn cast_ray(
        &mut self,
        ray: Ray3d,
        settings: &MeshRayCastSettings,
    ) -> &[(Entity, RayMeshHit)] {
        let ray_cull = info_span!("ray culling");
        let ray_cull_guard = ray_cull.enter();

        self.hits.clear();
        self.culled_list.clear();
        self.output.clear();

        // Check all entities to see if the ray intersects the AABB. Use this to build a short list
        // of entities that are in the path of the ray.
        let (aabb_hits_tx, aabb_hits_rx) = crossbeam_channel::unbounded::<(FloatOrd, Entity)>();
        let visibility_setting = settings.visibility;
        self.culling_query.par_iter().for_each(
            |(inherited_visibility, view_visibility, aabb, transform, entity)| {
                let should_ray_cast = match visibility_setting {
                    RayCastVisibility::Any => true,
                    RayCastVisibility::Visible => inherited_visibility.get(),
                    RayCastVisibility::VisibleInView => view_visibility.get(),
                };
                if should_ray_cast
                    && let Some(distance) = ray_aabb_intersection_3d(
                        ray,
                        &Aabb3d::new(aabb.center, aabb.half_extents),
                        &transform.affine(),
                    )
                {
                    aabb_hits_tx.send((FloatOrd(distance), entity)).ok();
                }
            },
        );
        *self.culled_list = aabb_hits_rx.try_iter().collect();

        // Sort by the distance along the ray.
        self.culled_list.sort_by_key(|(aabb_near, _)| *aabb_near);

        drop(ray_cull_guard);

        // Perform ray casts against the culled entities.
        let mut nearest_blocking_hit = FloatOrd(f32::INFINITY);
        let ray_cast_guard = debug_span!("ray_cast");
        self.culled_list
            .iter()
            .filter(|(_, entity)| (settings.filter)(*entity))
            .for_each(|(aabb_near, entity)| {
                // Get the mesh components and transform.
                let Ok((mesh2d, mesh3d, simplified_mesh, has_backfaces, transform)) =
                    self.mesh_query.get(*entity)
                else {
                    return;
                };

                // Get the underlying mesh handle. One of these will always be `Some` because of the query filters.
                let Some(mesh_handle) = simplified_mesh
                    .map(|m| &m.0)
                    .or(mesh3d.map(|m| &m.0).or(mesh2d.map(|m| &m.0)))
                else {
                    return;
                };

                // Is it even possible the mesh could be closer than the current best?
                if *aabb_near > nearest_blocking_hit {
                    return;
                }

                // Does the mesh handle resolve?
                let Some(mesh) = self.meshes.get(mesh_handle) else {
                    return;
                };

                // Backfaces of 2d meshes are never culled, unlike 3d meshes.
                let backfaces = match (has_backfaces, mesh2d.is_some()) {
                    (false, false) => Backfaces::Cull,
                    _ => Backfaces::Include,
                };

                // Perform the actual ray cast.
                let _ray_cast_guard = ray_cast_guard.enter();
                let transform = transform.affine();
                let bvh = self.cache.as_ref().and_then(|cache| {
                    cache.bvh(mesh_handle.id(), mesh, self.meshes.last_changed())
                });
                let intersection =
                    ray_intersection_over_mesh(mesh, &transform, ray, backfaces, bvh.as_deref());

                if let Some(intersection) = intersection {
                    let distance = FloatOrd(intersection.distance);
                    if (settings.early_exit_test)(*entity) && distance < nearest_blocking_hit {
                        // The reason we don't just return here is because right now we are
                        // going through the AABBs in order, but that doesn't mean that an
                        // AABB that starts further away can't end up with a closer hit than
                        // an AABB that starts closer. We need to keep checking AABBs that
                        // could possibly contain a nearer hit.
                        nearest_blocking_hit = distance.min(nearest_blocking_hit);
                    }
                    self.hits.push((distance, (*entity, intersection)));
                };
            });

        self.hits.retain(|(dist, _)| *dist <= nearest_blocking_hit);
        self.hits.sort_by_key(|(k, _)| *k);
        let hits = self.hits.iter().map(|(_, (e, i))| (*e, i.to_owned()));
        self.output.extend(hits);
        self.output.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{TaskPoolPlugin, Update};
    use bevy_asset::{AssetPlugin, RenderAssetUsages};
    use bevy_math::{ops, Affine3A, Dir3, Quat, Vec3};
    use bevy_mesh::{Indices, MeshBuilder, MeshPlugin, Meshable, PrimitiveTopology};
    use bevy_shape::{Sphere, Torus};

    /// A flat grid of `size` x `size` quads in the XZ plane, facing up.
    fn grid(size: u32) -> Mesh {
        let row = size + 1;
        let positions: Vec<[f32; 3]> = (0..row * row)
            .map(|i| [(i % row) as f32, 0.0, (i / row) as f32])
            .collect();
        let mut indices = Vec::new();
        for z in 0..size {
            for x in 0..size {
                let i = z * row + x;
                indices.extend([i, i + row, i + 1, i + 1, i + row, i + row + 1]);
            }
        }
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_indices(Indices::U32(indices))
    }

    #[test]
    fn bvh_gives_the_same_hits() {
        let meshes = [
            grid(20),
            Sphere::new(1.0).mesh().uv(32, 18),
            Torus::default().mesh().build(),
        ];
        let transform = Affine3A::from_scale_rotation_translation(
            Vec3::new(2.0, 1.0, 0.5),
            Quat::from_rotation_y(0.4),
            Vec3::new(1.0, 2.0, 3.0),
        );
        for mesh in &meshes {
            let (positions, indices) = mesh_triangles(mesh).unwrap();
            let bvh = build_bvh(positions, indices);
            for i in 0..200 {
                let angle = i as f32 * 0.37;
                let origin = transform.transform_point3(Vec3::new(
                    5.0 * ops::cos(angle),
                    3.0 + (i % 7) as f32 - 3.0,
                    5.0 * ops::sin(angle),
                ));
                let target = transform.transform_point3(Vec3::new(
                    (i % 5) as f32 * 0.4,
                    0.0,
                    (i % 3) as f32 * 0.3,
                ));
                let ray = Ray3d::new(origin, Dir3::new(target - origin).unwrap());
                for backfaces in [Backfaces::Cull, Backfaces::Include] {
                    let expected =
                        ray_intersection_over_mesh(mesh, &transform, ray, backfaces, None);
                    let actual =
                        ray_intersection_over_mesh(mesh, &transform, ray, backfaces, Some(&bvh));
                    assert_eq!(
                        expected
                            .as_ref()
                            .map(|hit| (hit.triangle_index, hit.distance, hit.point)),
                        actual
                            .as_ref()
                            .map(|hit| (hit.triangle_index, hit.distance, hit.point)),
                    );
                }
            }
        }
    }

    #[derive(Resource)]
    struct TestMesh(Handle<Mesh>);

    #[derive(Resource, Default)]
    struct TestHits(Vec<Option<f32>>);

    #[derive(Resource, Default)]
    struct MoveMesh(bool);

    /// Moves the test mesh 10 units along the X axis, if requested.
    fn move_mesh(
        mut move_mesh: ResMut<MoveMesh>,
        test_mesh: Res<TestMesh>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        if core::mem::take(&mut move_mesh.0) {
            meshes
                .get_mut(&test_mesh.0)
                .unwrap()
                .translate_by(Vec3::X * -10.0);
        }
    }

    /// Casts a ray down on the test mesh, after it was moved.
    fn cast_ray(mut ray_cast: MeshRayCast, mut hits: ResMut<TestHits>) {
        let ray = Ray3d::new(Vec3::new(103.3, 10.0, 4.7), Dir3::NEG_Y);
        let settings = MeshRayCastSettings::default().with_visibility(RayCastVisibility::Any);
        let hit = ray_cast
            .cast_ray(ray, &settings)
            .first()
            .map(|(_, hit)| hit.distance);
        hits.0.push(hit);
    }

    fn cache_entry(app: &App, test_mesh: &Handle<Mesh>) -> Option<&'static str> {
        let cache = app.world().resource::<MeshRayCastCache>();
        let entries = cache.entries.read().unwrap();
        entries.get(&test_mesh.id()).map(|entry| match entry {
            CacheEntry::CastOnce => "cast once",
            CacheEntry::Building(_) => "building",
            CacheEntry::Built(_) => "built",
        })
    }

    /// An app that casts a ray on a grid of `size` x `size` quads every frame.
    fn test_app(size: u32) -> (App, Handle<Mesh>) {
        let mut app = App::new();
        app.add_plugins((
            TaskPoolPlugin::default(),
            AssetPlugin::default(),
            MeshPlugin,
            MeshRayCastPlugin,
        ))
        .init_resource::<TestHits>()
        .init_resource::<MoveMesh>()
        .add_systems(Update, (move_mesh, cast_ray).chain());

        // Move the grid so that the ray hits it.
        let mesh = app
            .world_mut()
            .resource_mut::<Assets<Mesh>>()
            .add(grid(size).translated_by(Vec3::X * 100.0));
        app.insert_resource(TestMesh(mesh.clone()));
        app.world_mut().spawn((
            Mesh3d(mesh.clone()),
            Aabb::from_min_max(Vec3::new(0.0, -1.0, 0.0), Vec3::new(400.0, 1.0, 300.0)),
            GlobalTransform::IDENTITY,
            InheritedVisibility::VISIBLE,
            ViewVisibility::VISIBLE,
        ));
        (app, mesh)
    }

    /// Updates the app until the BVH of the mesh has been built in the background.
    fn update_until_built(app: &mut App, mesh: &Handle<Mesh>) {
        let start = std::time::Instant::now();
        while cache_entry(app, mesh) != Some("built") {
            assert!(matches!(
                cache_entry(app, mesh),
                Some("cast once" | "building")
            ));
            assert!(start.elapsed().as_secs() < 60, "the BVH was never built");
            std::thread::sleep(core::time::Duration::from_millis(1));
            app.update();
        }
    }

    #[test]
    fn large_meshes_are_built_in_the_background() {
        let (mut app, mesh) = test_app(192);

        app.update();
        app.update();
        assert_eq!(cache_entry(&app, &mesh), Some("cast once"));
        app.update();
        update_until_built(&mut app, &mesh);

        // Modify the mesh, and modify it again while its new BVH is being built: the BVH of the
        // intermediate mesh must never be used.
        app.world_mut().resource_mut::<MoveMesh>().0 = true;
        app.update();
        assert_eq!(cache_entry(&app, &mesh), None);
        app.update();
        assert_eq!(cache_entry(&app, &mesh), Some("cast once"));
        app.update();
        // The BVH may already have been built, if the task was fast enough.
        assert!(matches!(
            cache_entry(&app, &mesh),
            Some("building" | "built")
        ));
        app.world_mut().resource_mut::<MoveMesh>().0 = true;
        app.update();
        assert_eq!(cache_entry(&app, &mesh), None);
        app.update();
        app.update();
        update_until_built(&mut app, &mesh);
        for _ in 0..10 {
            app.update();
        }

        let hits = &app.world().resource::<TestHits>().0;
        assert!(hits.iter().all(|&hit| hit == Some(10.0)), "{hits:?}");
    }

    #[test]
    fn cache_is_built_and_invalidated() {
        let (mut app, mesh) = test_app(32);

        // The mesh was just added, so the cache can't be used.
        app.update();
        assert_eq!(cache_entry(&app, &mesh), None);
        // The BVH is built the second time the mesh is ray cast.
        app.update();
        assert_eq!(cache_entry(&app, &mesh), Some("cast once"));
        app.update();
        assert_eq!(cache_entry(&app, &mesh), Some("built"));
        app.update();
        assert_eq!(cache_entry(&app, &mesh), Some("built"));

        // Move the mesh, and cast the ray right after in the same frame: the ray still hits the
        // mesh, but not in any of the boxes of the BVH that was built before the move. The stale
        // BVH must not be used, even though the modification hasn't been processed yet.
        app.world_mut().resource_mut::<MoveMesh>().0 = true;
        app.update();
        assert_eq!(cache_entry(&app, &mesh), None);
        app.update();
        assert_eq!(cache_entry(&app, &mesh), Some("cast once"));
        app.update();
        assert_eq!(cache_entry(&app, &mesh), Some("built"));

        let hits = &app.world().resource::<TestHits>().0;
        assert_eq!(hits.len(), 7);
        assert!(hits.iter().all(|&hit| hit == Some(10.0)), "{hits:?}");
    }
}
