use bevy::picking::PickingSystems;
use bevy::picking::backend::ray::RayMap;
use bevy::picking::backend::{HitData, PointerHits};
use bevy::prelude::*;
use bevy_ecs_tilemap::anchor::TilemapAnchor;
use bevy_ecs_tilemap::map::{TilemapGridSize, TilemapSize, TilemapTileSize, TilemapType};
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};

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
    )>,
    mut pointer_hits_writer: MessageWriter<PointerHits>,
) {
    for (&ray_id, &ray) in ray_map.iter() {
        let Ok(camera) = picking_cameras.get(ray_id.camera) else {
            continue;
        };
        let mut hits = Vec::new();

        for (entity, map_size, grid_size, tile_size, map_type, storage, transform, anchor) in
            maps.iter()
        {
            let world_pos = (transform.to_matrix().inverse() * ray.origin.extend(1.)).xy();

            if let Some(tile_pos) = TilePos::from_world_pos(
                &world_pos, map_size, grid_size, tile_size, map_type, anchor,
            ) {
                if storage.get(&tile_pos).is_some() {
                    let pos: Vec2 = tile_pos.into();
                    let hit = HitData::new(ray_id.camera, 0., Some(pos.extend(0.)), None);
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
