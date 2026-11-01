use std::collections::VecDeque;

use godot::prelude::*;
use godot::classes::{CharacterBody2D, ICharacterBody2D};

use crate::nav::{MoveToAction, NavigationCapability};
use crate::{action::{ActionQueue, WaitAction}, nav::NavigationAgent, task::{Action, Capabilities, CapabilityProvider, TaskRunner}};

#[derive(GodotClass)]
#[class(base = CharacterBody2D, init)]
struct NPC {
    base: Base<CharacterBody2D>,

    #[init(node = "NavigationAgent2D")]
    nav_agent: OnReady<Gd<NavigationAgent>>,

    #[init(val = OnReady::manual())]
    task_runner: OnReady<TaskRunner>,
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

        godot_print!("assigning task");
        let host = self.base().clone().upcast();
        self.task_runner.assign_task(Box::new(task), host);
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