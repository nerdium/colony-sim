use std::collections::VecDeque;

use godot::prelude::*;
use godot::classes::{CharacterBody2D, ICharacterBody2D};

use crate::nav::{MoveToAction, NavigationCapability};
use crate::storage::{CELL_SIZE, FetchTask, ITEM_COLOR, Inventory, InventoryCapability, TransferItemAction};
use crate::world::World;
use crate::{action::{ActionQueue, WaitAction}, nav::NavigationAgent, task::{Action, Capabilities, TaskRunner}};

#[derive(GodotClass)]
#[class(base = CharacterBody2D, init)]
struct NPC {
    base: Base<CharacterBody2D>,

    #[init(node = "NavigationAgent2D")]
    nav_agent: OnReady<Gd<NavigationAgent>>,

    #[init(val = OnReady::manual())]
    task_runner: OnReady<TaskRunner>,

    #[init(val = Inventory::new(0))]
    inventory: Gd<Inventory>,

    #[export]
    world: OnEditor<Gd<World>>
}

impl NPC {
    fn capabilities(&self) -> Capabilities {
        let mut caps = Capabilities::default();
        
        caps.insert(NavigationCapability{ agent: self.nav_agent.clone() });
        caps.insert(InventoryCapability { inventory: self.inventory.clone() });

        caps
    }
}

#[godot_api]
impl ICharacterBody2D for NPC {
    fn ready(&mut self) {
        // Connected to the engine method, not a Rust handler: transfers run inside
        // physics_process, where NPC is already bound, so a handler would double-borrow.
        let redraw = self.base().callable("queue_redraw");
        self.inventory.connect("changed", &redraw);

        self.task_runner.init(TaskRunner::new(self.capabilities()));

        let world = self.world.bind();
        let storage_controller = world.storage_controller.bind();
        let inventory = storage_controller.find_with_item().unwrap().bind().inventory.clone();

        // let task = ActionQueue::new(VecDeque::from([
        //     Box::new(MoveToAction { target: Vector2i::new(5, 10) }) as Box<dyn Action>,
        //     Box::new(WaitAction { seconds: 5.0 }),
        //     Box::new(MoveToAction { target: Vector2i::new(5, 5) }),
        //     Box::new(TransferItemAction { from: inventory, to: self.inventory.clone() })
        // ]));
        
        let task = FetchTask::new(world.storage_controller.clone());

        let host = self.base().clone().upcast();
        self.task_runner.assign_task(Box::new(task), host);
    }

    fn draw(&mut self) {
        if self.inventory.bind().item_count > 0 {
            let size = Vector2::splat(CELL_SIZE);
            self.base_mut().draw_rect(Rect2::new(-size / 2.0, size), ITEM_COLOR);
        }
    }

    fn physics_process(&mut self, _delta: f64) {
        // advance tasks
        let host = self.base().clone().upcast();
        self.task_runner.poll(host);

        // navigation
        let body = &mut self.base().clone();
        self.nav_agent.bind_mut().advance(body);
    }
}