use std::collections::VecDeque;

use godot::{classes::{ISprite2D, Sprite2D}, prelude::*};

use crate::{action::{ActionQueue, Wait}, nav::MoveTo, task::{Action, TaskRunner}};

#[derive(GodotClass)]
#[class(base = Sprite2D, init)]
struct NPC {
    base: Base<Sprite2D>
}

#[godot_api]
impl ISprite2D for NPC {
    fn ready(&mut self) {
        let mut runner = TaskRunner::new_alloc();
        self.base_mut().add_child(&runner);

        let task = ActionQueue::new(VecDeque::from([
            Box::new(MoveTo { target: Vector2i::new(40, 20), seconds_per_tile: 1.0 }) as Box<dyn Action>,
            Box::new(Wait { seconds: 2.0 }),
            Box::new(MoveTo { target: Vector2i::new(10, 20), seconds_per_tile: 1.0 })
        ]));

        runner.bind_mut().start_task(Box::new(task));
    }
}