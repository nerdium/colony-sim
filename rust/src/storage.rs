use godot::classes::{INode2D, Node2D};
use godot::prelude::*;

use crate::nav::MoveToAction;
use crate::task::{Action, ActionCallback, ActionContext, ActionResult, Task};

const GRID_SIZE: i32 = 3;
const CAPACITY: i32 = GRID_SIZE * GRID_SIZE;

// Pixel layout of one 16x16 tile: 1px border and 1px gaps around 4x4 cells.
const TILE_SIZE: f32 = 16.0;
pub const CELL_SIZE: f32 = 4.0;
const CELL_GAP: f32 = 1.0;

const FRAME_COLOR: Color = Color::from_rgb(0.45, 0.45, 0.5);
const EMPTY_COLOR: Color = Color::from_rgb(0.12, 0.12, 0.15);
pub const ITEM_COLOR: Color = Color::from_rgb(0.95, 0.7, 0.2);

#[derive(GodotClass)]
#[class(base = RefCounted, init)]
pub struct Inventory {
    base: Base<RefCounted>,

    pub item_count: i32
}

#[godot_api]
impl Inventory {
    #[signal]
    fn changed();
}

impl Inventory {
    pub fn new(item_count: i32) -> Gd<Self> {
        Gd::from_init_fn(|base| Self { base, item_count })
    }

    pub fn add_item(&mut self) -> bool {
        if self.item_count >= CAPACITY {
            return false;
        }
        self.item_count += 1;
        self.signals().changed().emit();
        true
    }

    pub fn remove_item(&mut self) -> bool {
        if self.item_count <= 0 {
            return false;
        }
        self.item_count -= 1;
        self.signals().changed().emit();
        true
    }
}

pub struct InventoryCapability {
    pub inventory: Gd<Inventory>
}

/// Holds up to nine items, drawn as a 3x3 grid centered on the node's position.
/// Place the node at the center of a tile for it to fill that tile.
#[derive(GodotClass)]
#[class(base = Node2D, init)]
pub struct Storage {
    base: Base<Node2D>,
    #[init(val = Inventory::new_gd())]
    pub inventory: Gd<Inventory>,
    
    // TODO: move this to controller
    pub position: Vector2i
}

#[godot_api]
impl INode2D for Storage {
    fn ready(&mut self) {
        let redraw = self.base().callable("queue_redraw");
        self.inventory.connect("changed", &redraw);
    }

    fn draw(&mut self) {
        let half_tile = TILE_SIZE / 2.0;
        let item_count = self.inventory.bind().item_count;
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
    pub fn new(item_count: i32, position: Vector2i) -> Gd<Self> {
        let inventory = Inventory::new(item_count);
        Gd::from_init_fn(|base| Self { base, inventory, position })
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
    pub fn add_storage(&mut self, tile_pos: Vector2i, world_pos: Vector2, item_count: i32) {
        let mut storage = Storage::new(item_count, tile_pos);
        self.base_mut().add_child(&storage);
        storage.set_global_position(world_pos);

        self.storages.push(storage);
    }

    pub fn find_with_space(&self) -> Option<Gd<Storage>> {
        self.storages
            .iter()
            .find(|s| s.bind().inventory.bind().item_count < 9)
            .cloned()
    }

    pub fn find_with_item(&self) -> Option<Gd<Storage>> {
        self.storages
            .iter()
            .find(|s| s.bind().inventory.bind().item_count > 0)
            .cloned()
    }
}

pub enum TransferItemTarget {
    Provided(Gd<Inventory>),
    TaskRunner
}

impl TransferItemTarget {
    fn resolve(self, ctx: &ActionContext) -> Option<Gd<Inventory>> {
        match self {
            Self::Provided(inventory) => Some(inventory),
            Self::TaskRunner => ctx.caps
                .get::<InventoryCapability>()
                .map(|cap| cap.inventory.clone())
        }
    }
}

pub struct TransferItemAction {
    pub from: TransferItemTarget,
    pub to: TransferItemTarget
}

impl Action for TransferItemAction {
    fn start(self: Box<Self>, ctx: &mut ActionContext, result: ActionCallback) {
        // TODO: error handling?
        let mut from = self.from.resolve(ctx).unwrap();
        let mut to = self.to.resolve(ctx).unwrap();

        from.bind_mut().remove_item();
        to.bind_mut().add_item();
        result.success();
    }
}

enum FetchStage {
    MovingToStorage,
    TakingItem
}

pub struct FetchTask {
    storage_controller: Gd<StorageController>,
    inventory: Option<Gd<Inventory>>,
    stage: FetchStage
}

impl FetchTask {
    pub fn new(storage_controller: Gd<StorageController>) -> Self {
        Self {
            storage_controller,
            inventory: None,
            stage: FetchStage::MovingToStorage
        }
    }
}

impl Task for FetchTask {
    fn next(&mut self, prev: Option<ActionResult>) -> Option<Box<dyn Action>> {
        match (prev, &self.stage) {
            (None, _) => {
                let storage = self.storage_controller.bind().find_with_item()?;
                self.inventory = Some(storage.bind().inventory.clone());

                self.stage = FetchStage::MovingToStorage;
                Some(Box::new(MoveToAction { target: storage.bind().position }))
            },
            (Some(_), FetchStage::MovingToStorage) => {
                self.stage = FetchStage::TakingItem;
                Some(Box::new(TransferItemAction {
                    from: TransferItemTarget::Provided(self.inventory.clone().unwrap()),
                    to: TransferItemTarget::TaskRunner
                }))
            },
            (Some(_), FetchStage::TakingItem) => None
        }

    }
}