use std::collections::VecDeque;

use godot::prelude::*;

use crate::task::{Action, ActionResult, Completion, Task};

pub struct ActionQueue {
    queue: VecDeque<Box<dyn Action>>
}

impl ActionQueue {
    pub fn new(queue: VecDeque<Box<dyn Action>>) -> Self {
        Self { queue }
    }
}

impl Task for ActionQueue {
    fn next(&mut self, prev: Option<ActionResult>) -> Option<Box<dyn Action>> {
        match prev {
            Some(Err(err)) => godot_error!("ActionQueue failed: {}", err),
            _ => {}
        }
        
        self.queue.pop_front()
    }
}

pub struct Wait {
    pub seconds: f64
}

impl Action for Wait {
    fn start(self: Box<Self>, host: Gd<Node>, done: Completion) {
        let timer = host.get_tree().create_timer(self.seconds);
        godot::task::spawn(async move {
            timer.signals().timeout().to_future().await;
            godot_print!("waited for {} seconds", self.seconds);
            done.success();
        });
    }
}