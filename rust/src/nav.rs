use godot::classes::{CharacterBody2D, NavigationAgent2D, TileMapLayer};
use godot::prelude::*;

use crate::task::{Action, ActionCallback, ActionContext};

// TODO: bake navmesh



#[derive(GodotClass)]
#[class(base = NavigationAgent2D, init)]
pub struct NavigationAgent {
    base: Base<NavigationAgent2D>,
    target: Option<Vector2i>,
    move_callback: Option<ActionCallback>,

    #[export]
    tile_map: OnEditor<Gd<TileMapLayer>>,

    #[export]
    speed: f32
}

impl NavigationAgent {
    // pub fn new(speed: f32) -> Gd<Self> {
    //     Gd::from_init_fn(|base| Self {
    //         base,
    //         target: None,
    //         move_callback: None,
    //         speed
    //     })
    // }

    pub fn start_move(&mut self, target: Vector2i, callback: ActionCallback) {
        self.target = Some(target);
        self.move_callback = Some(callback);
        
        // TODO: 'to_global' can be removed if tilemap is at origin
        let world_pos = self.tile_map.map_to_local(target);
        let world_pos = self.tile_map.to_global(world_pos);

        let mut agent = self.base_mut();
        agent.set_target_position(world_pos);
    }

    pub fn advance(&mut self, body: &mut Gd<CharacterBody2D>) {
        if self.target.is_none() { return; }

        let next_pos = {
            let mut agent = self.base_mut();
            if agent.is_navigation_finished() {
                None
            } else {
                Some(agent.get_next_path_position())
            }
        };

        let Some(next_pos) = next_pos else {
            self.target = None;
            body.set_velocity(Vector2::ZERO);

            let result = self.move_callback.take().unwrap();
            result.success();
            return;
        };

        let curr_pos = body.get_global_position();
        body.set_velocity((next_pos - curr_pos).normalized_or_zero() * self.speed);
        body.move_and_slide();
    }
}

pub struct MoveToAction {
    pub target: Vector2i
}

impl Action for MoveToAction {
    fn start(self: Box<Self>, ctx: &mut ActionContext, result: ActionCallback) {
        let Some(nav_cap) = ctx.caps.get_mut::<NavigationCapability>() else {
            result.failure("host has no navigation capability");
            return;
        };

        let mut nav_agent = nav_cap.agent.bind_mut();

        nav_agent.start_move(self.target, result);
    }
}

pub struct NavigationCapability {
    pub agent: Gd<NavigationAgent>
}
