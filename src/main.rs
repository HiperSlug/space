mod picking;

use bevy::prelude::*;
use bevy_ecs_tilemap::anchor::TilemapAnchor;
use bevy_ecs_tilemap::map::{TilemapId, TilemapSize, TilemapTexture, TilemapTileSize, TilemapType};
use bevy_ecs_tilemap::tiles::{TileBundle, TileColor, TilePos, TileStorage};
use bevy_ecs_tilemap::{TilemapBundle, TilemapPlugin};

use picking::TilemapPickingPlugin;

fn main() {
    App::new().add_plugins(ProductivePackerDeluxe).run();
}

struct ProductivePackerDeluxe;

impl Plugin for ProductivePackerDeluxe {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins);
        app.add_plugins((TilemapPlugin, TilemapPickingPlugin));
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let size = TilemapSize::new(16, 16);

    let tilemap_id = commands.spawn_empty().id();

    let mut storage = TileStorage::empty(size);

    for x in 0..size.x {
        for y in 0..size.y {
            let position = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position,
                    tilemap_id: TilemapId(tilemap_id),
                    color: TileColor(Color::srgb(x as f32 / 16., y as f32 / 16., 1.)),
                    ..default()
                })
                .id();
            storage.set(&position, tile_entity);
        }
    }

    let tile_size = TilemapTileSize::new(16., 16.);
    let grid_size = tile_size.into();
    let map_type = TilemapType::Square;

    // tilemap
    commands
        .entity(tilemap_id)
        .insert((TilemapBundle {
            tile_size,
            grid_size,
            map_type,
            size,
            storage,
            texture: TilemapTexture::Single(asset_server.load("tileset.png")),
            anchor: TilemapAnchor::Center,
            ..default()
        },))
        .observe(on_drag_follow);

    // player
    commands.spawn(Camera2d);
}

fn on_drag_follow(drag: On<Pointer<Drag>>, mut transforms: Query<&mut Transform>) {
    if let Ok(mut transform) = transforms.get_mut(drag.entity) {
        let mut delta = drag.delta.extend(0.);
        delta.y *= -1.;
        transform.translation += delta;
    }
}
