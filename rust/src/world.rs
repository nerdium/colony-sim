use godot::{classes::{INavigationRegion2D, NavigationMeshSourceGeometryData2D, NavigationPolygon, NavigationRegion2D, NavigationServer2D, TileMapLayer, navigation_polygon::ParsedGeometryType}, prelude::*};

use crate::storage::{StorageController};

#[derive(GodotClass)]
#[class(base = NavigationRegion2D, init)]
pub struct World {
    base: Base<NavigationRegion2D>,

    #[export]
    tilemap: OnEditor<Gd<TileMapLayer>>,
    
    #[export]
    agent_radius: f32,
    
    #[init(node = "StorageController")]
    pub storage_controller: OnReady<Gd<StorageController>>,

    dirty: bool
}

#[godot_api]
impl INavigationRegion2D for World {
    fn ready(&mut self) {
        self.tilemap.set_navigation_enabled(false);
        self.rebake();

        let storage_tile = Vector2i::new(0, 0);
        let storage_pos = self.tile_to_world_pos(storage_tile);
        self.storage_controller.bind_mut().add_storage(storage_tile, storage_pos, 8);

        let storage_tile = Vector2i::new(0, 20);
        let storage_pos = self.tile_to_world_pos(storage_tile);
        self.storage_controller.bind_mut().add_storage(storage_tile, storage_pos, 1);
    }

    fn process(&mut self, _dr: f64) {
        if self.dirty {
            self.dirty = false;
            self.rebake();
        }
    }
}

impl World {
    pub fn tile_to_world_pos(&self, tile_pos: Vector2i) -> Vector2 {
        self.tilemap.to_global(self.tilemap.map_to_local(tile_pos))
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn rebake(&mut self) {
        let Some(tileset) = self.tilemap.get_tile_set() else { return; };

        let mut nav_poly = NavigationPolygon::new_gd();
        nav_poly.set_agent_radius(self.agent_radius);
        nav_poly.set_parsed_geometry_type(ParsedGeometryType::STATIC_COLLIDERS);
        nav_poly.set_parsed_collision_mask(1);

        let used = self.tilemap.get_used_rect();
        let tile_size = tileset.get_tile_size();

        let min = (used.position * tile_size).to_vector2();
        let max = ((used.position + used.size) * tile_size).to_vector2();

        let outline: PackedVector2Array = [
            Vector2::new(min.x, min.y),
            Vector2::new(max.x, min.y),
            Vector2::new(max.x, max.y),
            Vector2::new(min.x, max.y)
        ]
        .into_iter()
        .collect();
        nav_poly.add_outline(&outline);
    
        let source = NavigationMeshSourceGeometryData2D::new_gd();
        let mut nav = NavigationServer2D::singleton();
        let node = self.tilemap.clone().upcast::<Node>();

        nav.parse_source_geometry_data(&nav_poly, &source, &node);
        nav.bake_from_source_geometry_data_async(&nav_poly, &source);
        
        self.base_mut().set_navigation_polygon(&nav_poly);
    }
}