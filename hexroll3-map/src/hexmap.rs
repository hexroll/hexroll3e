use std::collections::HashMap;

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        view::RenderLayers,
    },
};
use hexx::{shapes, Hex, HexLayout, HexOrientation, OffsetHexMode, PlaneMeshBuilder};

pub struct HexMap;

impl Plugin for HexMap {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, update_hex_map_tiles)
            .add_systems(Update, hex_map_zoom_dimmer);
    }
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Debug, Resource)]
struct HexMapData {
    pub cmin: Hex,
    pub cmax: Hex,
}

#[derive(Debug, Resource)]
struct HexMapResources {
    pub mesh: Handle<Mesh>,
    pub forest_main_tile_material: Handle<StandardMaterial>,
    pub forest_dungeon_tile_material: Handle<StandardMaterial>,
}

#[derive(Component)]
struct HexEntity {
    hex: Hex,
}

const HEX_SIZE: Vec2 = Vec2::splat(120.0);

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    let layout = HexLayout {
        orientation: HexOrientation::Flat,
        hex_size: HEX_SIZE,
        ..default()
    };
    let forest_main_tile_texture = asset_server.load("forest.ktx2");
    let forest_dungeon_tile_texture = asset_server.load("forest-dungeon.ktx2");
    let color = Color::srgb(0.3, 0.5, 0.1);

    let forest_main_tile_material = materials.add(StandardMaterial {
        base_color_texture: Some(forest_main_tile_texture.clone()),
        emissive_texture: Some(forest_main_tile_texture.clone()),
        emissive: color.into(),
        ..default()
    });
    let forest_dungeon_tile_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.7, 0.1),
        base_color_texture: Some(forest_dungeon_tile_texture.clone()),
        alpha_mode: AlphaMode::Opaque,
        emissive_texture: Some(forest_dungeon_tile_texture.clone()),
        emissive: color.into(),
        ..default()
    });

    commands.insert_resource(HexMapResources {
        mesh: meshes.add(hexagonal_plane(&layout)),
        forest_main_tile_material: forest_main_tile_material.clone(),
        forest_dungeon_tile_material: forest_dungeon_tile_material.clone(),
    });
    commands.insert_resource(HexMapData {
        cmin: Hex::ZERO,
        cmax: Hex::ZERO,
    });
}

fn hex_map_zoom_dimmer(
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_projection: Query<&Projection, With<MainCamera>>,
    assets: ResMut<HexMapResources>,
) {
    let proj = camera_projection.single();
    if let Projection::Orthographic(proj) = proj {
        let ratio = 1.0 - ((proj.scale - 500.0) / 1000.0).clamp(0.0, 1.0);
        materials
            .get_mut(&assets.forest_main_tile_material)
            .unwrap()
            .emissive = Color::srgb(1.0, 1.0, 1.0).darker(ratio).into();
        materials
            .get_mut(&assets.forest_dungeon_tile_material)
            .unwrap()
            .emissive = Color::srgb(1.0, 1.0, 1.0).darker(ratio).into();
    }
}

fn update_hex_map_tiles(
    mut commands: Commands,
    hexes: Query<(Entity, &HexEntity)>,
    cameras: Query<(&GlobalTransform, &Projection), With<MainCamera>>,
    map_resources: Res<HexMapResources>,
    mut map_data: ResMut<HexMapData>,
) {
    let (cam_transform, proj) = cameras.single();
    let layout = HexLayout {
        orientation: HexOrientation::Flat,
        hex_size: HEX_SIZE,
        ..default()
    };
    if let Projection::Orthographic(proj) = proj {
        if proj.scale > 50000.0 {
            return;
        }
        let view_size_halved = (proj.area.max - proj.area.min) / 2.0;
        let cmin = layout.world_pos_to_hex(cam_transform.translation().xz() - view_size_halved);
        let cmax = layout.world_pos_to_hex(cam_transform.translation().xz() + view_size_halved);
        if cmin == map_data.cmin && cmax == map_data.cmax {
            return;
        }
        map_data.cmin = cmin;
        map_data.cmax = cmax;

        let mut hex_map: HashMap<_, _> = hexes.iter().map(|(e, h)| (h.hex, e)).collect();

        let [cmin_x, cmin_y] = cmin.to_offset_coordinates(OffsetHexMode::EvenColumns);
        let [cmax_x, cmax_y] = cmax.to_offset_coordinates(OffsetHexMode::EvenColumns);

        shapes::flat_rectangle([cmin_x - 1, cmax_x + 1, cmax_y - 1, cmin_y]).for_each(|hex| {
            if !hex_map.contains_key(&hex) {
                let pos = layout.hex_to_world_pos(hex);
                commands.spawn_empty().insert((
                    HexEntity { hex },
                    Mesh3d(map_resources.mesh.clone()),
                    MeshMaterial3d(get_tile_material(hex, &map_resources)),
                    RenderLayers::layer(0),
                    Transform::from_xyz(pos.x, 0.0, pos.y),
                ));
            }
            hex_map.remove(&hex);
        });
        for value in hex_map.values() {
            commands.entity(*value).despawn_recursive();
        }
    }
}

fn get_tile_material(hex: Hex, map_resources: &Res<HexMapResources>) -> Handle<StandardMaterial> {
    // The prototype map is one big forest with a single dungeon in the middle:
    if hex.x == 0 && hex.y == 0 {
        map_resources.forest_dungeon_tile_material.clone()
    } else {
        map_resources.forest_main_tile_material.clone()
    }
}

fn hexagonal_plane(hex_layout: &HexLayout) -> Mesh {
    let mesh_info = PlaneMeshBuilder::new(hex_layout)
        .facing(Vec3::Y)
        .with_scale(Vec3::splat(0.95))
        .center_aligned()
        .build();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs)
    .with_inserted_indices(Indices::U16(mesh_info.indices))
}
