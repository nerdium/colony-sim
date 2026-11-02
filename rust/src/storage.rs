use godot::classes::{INode2D, Node2D};
use godot::prelude::*;

use crate::world::World;

const GRID_SIZE: i32 = 3;
const CAPACITY: i32 = GRID_SIZE * GRID_SIZE;

// Pixel layout of one 16x16 tile: 1px border and 1px gaps around 4x4 cells.
const TILE_SIZE: f32 = 16.0;
pub const CELL_SIZE: f32 = 4.0;
const CELL_GAP: f32 = 1.0;

const FRAME_COLOR: Color = Color::from_rgb(0.45, 0.45, 0.5);
const EMPTY_COLOR: Color = Color::from_rgb(0.12, 0.12, 0.15);
pub const ITEM_COLOR: Color = Color::from_rgb(0.95, 0.7, 0.2);

/// Holds up to nine items, drawn as a 3x3 grid centered on the node's position.
/// Place the node at the center of a tile for it to fill that tile.
#[derive(GodotClass)]
#[class(base = Node2D, init, tool)]
pub struct Storage {
    base: Base<Node2D>,

    #[export(range = (0.0, 9.0))]
    #[var(set = set_item_count)]
    pub item_count: i32,
}

#[godot_api]
impl INode2D for Storage {
    fn draw(&mut self) {
        let half_tile = TILE_SIZE / 2.0;
        let item_count = self.item_count;
        let mut node = self.base_mut();

        node.draw_rect(
            Rect2::new(Vector2::splat(-half_tile), Vector2::splat(TILE_SIZE)),
            FRAME_COLOR,
        );

        // Items fill the grid left to right, top to bottom.
        for slot in 0..CAPACITY {
            let column = (slot % GRID_SIZE) as f32;
            let row = (slot / GRID_SIZE) as f32;
            let offset = CELL_SIZE + CELL_GAP;
            let position = Vector2::new(
                -half_tile + CELL_GAP + column * offset,
                -half_tile + CELL_GAP + row * offset,
            );
            let color = if slot < item_count { ITEM_COLOR } else { EMPTY_COLOR };

            node.draw_rect(Rect2::new(position, Vector2::splat(CELL_SIZE)), color);
        }
    }
}

impl Storage {
    pub fn new(item_count: i32) -> Gd<Self> {
        Gd::from_init_fn(|base| Self { base, item_count })
    }
}

#[godot_api]
impl Storage {
    #[func]
    pub fn set_item_count(&mut self, count: i32) {
        self.item_count = count.clamp(0, CAPACITY);
        self.base_mut().queue_redraw();
    }

    /// Adds one item, returning false if the storage is already full.
    #[func]
    pub fn add_item(&mut self) -> bool {
        if self.item_count >= CAPACITY {
            return false;
        }
        self.set_item_count(self.item_count + 1);
        true
    }

    /// Removes one item, returning false if the storage is already empty.
    #[func]
    pub fn remove_item(&mut self) -> bool {
        if self.item_count <= 0 {
            return false;
        }
        self.set_item_count(self.item_count - 1);
        true
    }
}

#[derive(GodotClass)]
#[class(base = Node2D, init)]
pub struct StorageController {
    base: Base<Node2D>,

    storages: Vec<Gd<Storage>>
}

impl StorageController {
    // TODO: take tile pos instead of world pos
    pub fn add_storage(&mut self, world_pos: Vector2, item_count: i32) {
        let mut storage = Storage::new(item_count);
        self.base_mut().add_child(&storage);
        storage.set_global_position(world_pos);

        self.storages.push(storage);
    }

    pub fn find_with_space(&self) -> Option<Gd<Storage>> {
        self.storages
            .iter()
            .find(|s| s.bind().item_count < 9)
            .cloned()
    }

    pub fn find_with_item(&self) -> Option<Gd<Storage>> {
        self.storages
            .iter()
            .find(|s| s.bind().item_count > 0)
            .cloned()
    }
}