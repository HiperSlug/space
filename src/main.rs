mod picking;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use ndshape::{RuntimeShape, Shape};
use std::{ops::Index, sync::Arc};

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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut me: ResMut<Assets<Mesh>>, mut ma: ResMut<Assets<ColorMaterial>>) {
    let square_2x2 = Arc::new(Pattern::from([[true; 2]; 2]));
    let square_5x5 = Arc::new(Pattern::from([[true; 5]; 5]));

    // background
    let size = TilemapSize::from(UVec2::from(square_2x2.shape.as_array()));
    let mut storage = TileStorage::empty(size);
    let entity = commands.spawn_empty().id();
    fill_tilemap(
        TileTextureIndex(1),
        size,
        TilemapId(entity),
        &mut commands,
        &mut storage,
    );
    commands.entity(entity).insert((
        TilemapBundle {
            size,
            storage,
            tile_size: TilemapTileSize::new(16., 16.),
            grid_size: TilemapGridSize::new(16., 16.),
            texture: TilemapTexture::Single(asset_server.load("tileset.png")),
            anchor: TilemapAnchor::Center,
            ..default()
        },
    ));

    // package
    commands.spawn((
        TilemapBundle {
            size: TilemapSize::from(UVec2::splat(5)),
            tile_size: TilemapTileSize::new(16., 16.),
            grid_size: TilemapGridSize::new(16., 16.),
            texture: TilemapTexture::Single(asset_server.load("tileset.png")),
            anchor: TilemapAnchor::Center,
            ..default()
        },
        Package(square_5x5),
        Mesh2d(me.add(Circle::new(17.))),
        MeshMaterial2d(ma.add(Color::WHITE)),
    )).add_child(entity);

    // piece
    let size = TilemapSize::from(UVec2::from(square_2x2.shape.as_array()));
    let mut storage = TileStorage::empty(size);
    let entity = commands.spawn_empty().id();
    fill_tilemap(
        TileTextureIndex(0),
        size,
        TilemapId(entity),
        &mut commands,
        &mut storage,
    );
    commands
        .entity(entity)
        .insert((
            TilemapBundle {
                size,
                storage,
                tile_size: TilemapTileSize::new(16., 16.),
                grid_size: TilemapGridSize::new(16., 16.),
                texture: TilemapTexture::Single(asset_server.load("tileset.png")),
                anchor: TilemapAnchor::Center,
                ..default()
            },
            Piece(square_2x2),
        ))
        .observe(on_drag_follow)
        .observe(on_drag_drop_info);

    // camera
    commands.spawn(Camera2d);
}

#[derive(Deref)]
struct Pattern {
    shape: RuntimeShape<u32, 2>,
    #[deref]
    pattern: Vec<bool>,
}

impl<const X: usize, const Y: usize> From<[[bool; X]; Y]> for Pattern {
    fn from(value: [[bool; X]; Y]) -> Self {
        let shape = RuntimeShape::<u32, 2>::new([X as u32, Y as u32]);
        let pattern = value.into_iter().flatten().collect();
        Self { shape, pattern }
    }
}

impl<P: Into<[u32; 2]>> Index<P> for Pattern {
    type Output = bool;

    fn index(&self, pos: P) -> &Self::Output {
        let arr = pos.into();
        let i = self.shape.linearize(arr);
        &self.pattern[i as usize]
    }
}

#[derive(Component, Deref)]
struct Package(Arc<Pattern>);

#[derive(Component, Deref)]
struct Piece(Arc<Pattern>);

fn on_drag_follow(drag: On<Pointer<Drag>>, mut transforms: Query<&mut Transform>) {
    if let Ok(mut transform) = transforms.get_mut(drag.entity) {
        let mut delta = drag.delta.extend(0.);
        delta.y *= -1.;
        transform.translation += delta;
    }
}

fn on_drag_drop_info(drag_drop: On<Pointer<DragDrop>>) {
    info!("{} -> {}", drag_drop.dropped, drag_drop.event_target());
}

// fn on_drag_drop_combine(
//     drag_drop: On<Pointer<DragDrop>>,
//     mut commands: Commands,
//     mut map_query: Query<(
//         Entity,
//         &TilemapSize,
//         &TilemapGridSize,
//         &TilemapTileSize,
//         &TilemapType,
//         &mut TileStorage,
//         &TilemapAnchor,
//     )>,
// ) {
//     let Some(world_pos) = drag_drop.event.hit.position else {
//         return;
//     };
//     let world_pos = world_pos.xy();

//     let Ok([src, dst]) = map_query.get_many_mut([drag_drop.event.dropped, drag_drop.entity]) else {
//         return;
//     };

//     let (src_entity, src_tile, src_storage, src_size) = {
//         let (entity, map_size, grid_size, tile_size, map_type, storage, anchor) = src;
//         (
//             entity,
//             TilePos::from_world_pos(&world_pos, map_size, grid_size, tile_size, map_type, anchor)
//                 .unwrap(),
//             storage,
//             map_size,
//         )
//     };
//     let (dst_tile, mut dst_storage) = {
//         let (_, map_size, grid_size, tile_size, map_type, storage, anchor) = dst;
//         (
//             TilePos::from_world_pos(&world_pos, map_size, grid_size, tile_size, map_type, anchor)
//                 .unwrap(),
//             storage,
//         )
//     };

//     let delta = UVec2::from(dst_tile).as_ivec2() - UVec2::from(src_tile).as_ivec2();

//     for y in 0..src_size.y {
//         for x in 0..src_size.x {
//             let pos = uvec2(x, y);
//             let src_tile_pos: TilePos = pos.into();
//             let dst_tile_pos: TilePos = pos.wrapping_add_signed(delta).into();
//             let src_occupied = src_storage.checked_get(&src_tile_pos).is_some();
//             let dst_ob = !dst_tile_pos.within_map_bounds(&dst_storage.size);
//             let dst_occupied = dst_storage.checked_get(&dst_tile_pos).is_some();

//             if src_occupied & (dst_occupied | dst_ob) {
//                 // HANDLE FAILURE
//                 return;
//             }
//         }
//     }

//     // a little ridiculous there is no native way to iterate over (position, entity)
//     let src_storage = &*src_storage;
//     let src_iter = (0..src_size.y).flat_map(|y| {
//         (0..src_size.x).filter_map(move |x| {
//             let tile_pos = TilePos::new(x, y);
//             src_storage.get(&tile_pos).map(|e| (uvec2(x, y), e))
//         })
//     });

//     for (src_pos, entity) in src_iter {
//         let dst_pos = src_pos.wrapping_add_signed(delta);
//         dst_storage.set(&dst_pos.into(), entity);
//     }

//     // PERHAPS DONT DESPAWN
//     commands.entity(src_entity).despawn();
// }
