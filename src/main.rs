mod picking;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use ndshape::{RuntimeShape, Shape};
use std::{ops::Index, sync::Arc};

use picking::{PickableTilemap, TilemapPickingPlugin};

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
    let square_2x2 = Arc::new(Pattern::from([[true; 2]; 2]));
    let square_5x5 = Arc::new(Pattern::from([[true; 5]; 5]));

    let texture = TilemapTexture::Single(asset_server.load("tileset.png"));
    let tile_size = TilemapTileSize::new(64., 64.);
    let grid_size = TilemapGridSize::new(64., 64.);
    let anchor = TilemapAnchor::Center;

    // background
    let size = TilemapSize::from(UVec2::from(square_5x5.shape.as_array()));
    let mut storage = TileStorage::empty(size);
    let background_entity = commands.spawn_empty().id();
    fill_tilemap(
        TileTextureIndex(3),
        size,
        TilemapId(background_entity),
        &mut commands,
        &mut storage,
    );
    commands.entity(background_entity).insert(TilemapBundle {
        size,
        storage,
        tile_size,
        grid_size,
        texture: texture.clone(),
        anchor,
        ..default()
    });

    // package
    let storage = TileStorage::empty(size);
    commands
        .spawn((
            TilemapBundle {
                size,
                storage,
                tile_size,
                grid_size,
                texture: texture.clone(),
                anchor,
                ..default()
            },
            Package(square_5x5),
            PickableTilemap::new(false, -1),
        ))
        .add_child(background_entity)
        .observe(|drag_drop: On<Pointer<DragDrop>>| {
            info!("{} -> {}", drag_drop.dropped, drag_drop.entity)
        }).observe(on_drag_drop_combine);

    // piece
    let origin = vec2(200., 10.);
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
    commands
        .entity(entity)
        .insert((
            Piece(square_2x2),
            PieceOrigin(origin),
            TilemapBundle {
                size,
                storage,
                tile_size,
                grid_size,
                texture: texture.clone(),
                anchor,
                transform: Transform::from_translation(origin.extend(0.)),
                ..default()
            },
            PickableTilemap::new(true, 0),
        ))
        .observe(
            |drag_start: On<Pointer<DragStart>>, mut query: Query<&mut PickableTilemap>| {
                query.get_mut(drag_start.entity).unwrap().priority = i32::MIN;
            },
        )
        .observe(
            |drag: On<Pointer<Drag>>, mut query: Query<&mut Transform>| {
                query.get_mut(drag.entity).unwrap().translation +=
                    drag.delta.extend(0.) * vec3(1., -1., 1.);
            },
        )
        .observe(
            |drag_end: On<Pointer<DragEnd>>,
             mut query: Query<(&mut Transform, &mut PickableTilemap, &PieceOrigin)>| {
                info!("a");
                let (mut transform, mut pickable_settings, origin) =
                    query.get_mut(drag_end.entity).unwrap();
                pickable_settings.priority = 0;
                transform.translation = origin.0.extend(0.);
            },
        );

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

impl<const X: usize, const Y: usize> From<[[u8; X]; Y]> for Pattern {
    fn from(value: [[u8; X]; Y]) -> Self {
        let bools = value.map(|arr| arr.map(|byte| byte & 1 != 0));
        Self::from(bools)
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

#[derive(Component, Default)]
struct PieceOrigin(Vec2);

fn on_drag_drop_combine(
    drag_drop: On<Pointer<DragDrop>>,
    mut map_query: Query<(
        Entity,
        &TilemapSize,
        &TilemapGridSize,
        &TilemapTileSize,
        &TilemapType,
        &mut TileStorage,
        &TilemapAnchor,
        &Transform,
    )>,
    mut tile_query: Query<(&mut TilemapId, &mut TilePos, &mut TilePosOld)>,
    package_query: Query<&Package>,
    mut piece_query: Query<&mut Piece>,
) {
    info!("b");
    let Some(world_pos) = drag_drop.hit.position else {
        return;
    };
    let world_pos = world_pos.xy();

    let Ok([src, dst]) = map_query.get_many_mut([drag_drop.dropped, drag_drop.entity]) else {
        return;
    };

    let Ok(package) = package_query.get(drag_drop.entity) else {
        return;
    };

    let Ok(piece) = piece_query.get_mut(drag_drop.dropped) else {
        return;
    };

    let src_storage;
    let mut dst_storage;

    let dst_entity;

    let delta = {
        let Some(src_tile) = ({
            let (_, map_size, grid_size, tile_size, map_type, storage, anchor, transform) = src;
            src_storage = storage;
            let model_pos = transform.to_matrix().inverse() * world_pos.extend(0.).extend(1.);
            TilePos::from_world_pos(&model_pos.xy(), map_size, grid_size, tile_size, map_type, anchor)
        }) else {
            error!("src");
            return;
        };

        let Some(dst_tile) = ({
            let (entity, map_size, grid_size, tile_size, map_type, storage, anchor, transform) = dst;
            dst_storage = storage;
            dst_entity = entity;
            let model_pos = transform.to_matrix().inverse() * world_pos.extend(0.).extend(1.);
            TilePos::from_world_pos(&model_pos.xy(), map_size, grid_size, tile_size, map_type, anchor)
        }) else {
            error!("dst");
            return;
        };
        UVec2::from(dst_tile).as_ivec2() - UVec2::from(src_tile).as_ivec2()
    };

    for y in 0..src_storage.size.y {
        for x in 0..src_storage.size.x {
            let src_pos = uvec2(x, y);
            let dst_pos = src_pos.wrapping_add_signed(delta);

            let dst_ob = !TilePos::from(dst_pos).within_map_bounds(&dst_storage.size);
            if dst_ob {
                // failure
                return;
            }

            let src_occupied = src_storage.get(&src_pos.into()).is_some();
            let dst_occupied = dst_storage.get(&dst_pos.into()).is_some();
            let good_pattern = package[dst_pos];

            if src_occupied & dst_occupied & good_pattern {
                // failure
                return;
            }
        }
    }

    // a little ridiculous there is no native way to iterate over (position, entity)
    let src_storage = &*src_storage;
    let src_iter = (0..src_storage.size.y).flat_map(|y| {
        (0..src_storage.size.x).filter_map(move |x| {
            let tile_pos = TilePos::new(x, y);
            src_storage.get(&tile_pos).map(|e| (uvec2(x, y), e))
        })
    });

    for (src_tile, entity) in src_iter {
        let dst_tile = src_tile.wrapping_add_signed(delta).into();
        dst_storage.set(&dst_tile, entity);
        let (mut id, mut pos, mut pos_old) = tile_query.get_mut(entity).unwrap();
        id.0 = dst_entity;
        *pos = dst_tile;
        *pos_old = TilePosOld(dst_tile);
    }

    // TODO
    let _ = piece;
}
