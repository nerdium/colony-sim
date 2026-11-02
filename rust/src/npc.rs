use std::collections::VecDeque;

use godot::prelude::*;
use godot::classes::{CharacterBody2D, ICharacterBody2D};

use crate::nav::{MoveToAction, NavigationCapability};
use crate::storage::{CELL_SIZE, ITEM_COLOR};
use crate::{action::{ActionQueue, WaitAction}, nav::NavigationAgent, task::{Action, Capabilities, CapabilityProvider, TaskRunner}};

#[derive(GodotClass)]
#[class(base = CharacterBody2D, init)]
struct NPC {
    base: Base<CharacterBody2D>,

    #[init(node = "NavigationAgent2D")]
    nav_agent: OnReady<Gd<NavigationAgent>>,

    #[init(val = OnReady::manual())]
    task_runner: OnReady<TaskRunner>,

    /// The NPC's single item slot, drawn as a square on the NPC while filled.
    #[export]
    #[var(set = set_has_item)]
    has_item: bool
}

#[godot_api]
impl NPC {
    #[func]
    pub fn set_has_item(&mut self, has_item: bool) {
        self.has_item = has_item;
        self.base_mut().queue_redraw();
    }

    /// Fills the slot, returning false if it already holds an item.
    #[func]
    pub fn add_item(&mut self) -> bool {
        if self.has_item {
            return false;
        }
        self.set_has_item(true);
        true
    }

    /// Empties the slot, returning false if it holds no item.
    #[func]
    pub fn remove_item(&mut self) -> bool {
        if !self.has_item {
            return false;
        }
        self.set_has_item(false);
        true
    }
}

#[godot_api]
impl ICharacterBody2D for NPC {
    fn ready(&mut self) {
        let cap_provider = NPCCapabilityProvider {
            nav_agent: (*self.nav_agent).clone()
        };
        self.task_runner.init(TaskRunner::new(cap_provider));

        let task = ActionQueue::new(VecDeque::from([
            Box::new(MoveToAction { target: Vector2i::new(40, 20) }) as Box<dyn Action>,
            Box::new(WaitAction { seconds: 10.0 }),
            Box::new(MoveToAction { target: Vector2i::new(10, 20) })
        ]));

        let host = self.base().clone().upcast();
        self.task_runner.assign_task(Box::new(task), host);
    }

    fn draw(&mut self) {
        if self.has_item {
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

struct NPCCapabilityProvider {
    pub nav_agent: Gd<NavigationAgent>
}

impl CapabilityProvider for NPCCapabilityProvider {
    fn provide(&self, caps: &mut Capabilities) {
        caps.insert(NavigationCapability{ agent: self.nav_agent.clone() });
    }
}