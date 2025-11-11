//! Based on
//! [MeshPickingPlugin](https://github.com/bevyengine/bevy/blob/main/crates/bevy_picking/src/mesh_picking/mod.rs#L67)
//! & [mouse_to_tile.rs](https://github.com/StarArawn/bevy_ecs_tilemap/blob/main/examples/mouse_to_tile.rs)

use bevy::picking::PickingSystems;
use bevy::picking::backend::ray::RayMap;
use bevy::picking::backend::{HitData, PointerHits};
use bevy::prelude::*;
use bevy_ecs_tilemap::anchor::TilemapAnchor;
use bevy_ecs_tilemap::map::{TilemapGridSize, TilemapSize, TilemapTileSize, TilemapType};
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};

#[derive(Component, Default)]
pub struct PickableTilemap {
    pub occupied_only: bool,
    pub priority: i32,
}

impl PickableTilemap {
    pub fn new(occupied_only: bool, priority: i32) -> Self {
        Self {
            occupied_only,
            priority,
        }
    }
}
pub struct TilemapPickingPlugin;

impl Plugin for TilemapPickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, update_hits.in_set(PickingSystems::Backend));
    }
}

pub fn update_hits(
    ray_map: Res<RayMap>,
    picking_cameras: Query<&Camera>,
    maps: Query<(
        Entity,
        &TilemapSize,
        &TilemapGridSize,
        &TilemapTileSize,
        &TilemapType,
        &TileStorage,
        &GlobalTransform,
        &TilemapAnchor,
        &PickableTilemap,
    )>,
    mut pointer_hits_writer: MessageWriter<PointerHits>,
) {
    for (&ray_id, &ray) in ray_map.iter() {
        let Ok(camera) = picking_cameras.get(ray_id.camera) else {
            continue;
        };
        let mut hits = Vec::new();

        for (
            entity,
            map_size,
            grid_size,
            tile_size,
            map_type,
            storage,
            transform,
            anchor,
            settings,
        ) in maps.iter()
        {
            let tilemap_plane_z = transform.translation().z;
            let t = (tilemap_plane_z - ray.origin.z) / ray.direction.z;
            let world_pos = ray.origin + ray.direction.as_vec3() * t;

            let model_pos = (transform.to_matrix().inverse() * ray.origin.extend(1.)).xy();

            if let Some(tile_pos) = TilePos::from_world_pos(
                &model_pos, map_size, grid_size, tile_size, map_type, anchor,
            ) {
                if !settings.occupied_only || storage.get(&tile_pos).is_some() {
                    let hit = HitData::new(
                        ray_id.camera,
                        -(settings.priority as f32),
                        Some(world_pos),
                        None,
                    );
                    hits.push((entity, hit));
                }
            }
        }

        if !hits.is_empty() {
            let order = camera.order as f32;
            pointer_hits_writer.write(PointerHits::new(ray_id.pointer, hits, order));
        }
    }
}
