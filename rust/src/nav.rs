use godot::classes::a_star_grid_2d::DiagonalMode;
use godot::classes::{AStarGrid2D, TileMapLayer};
use godot::prelude::*;

use crate::task::{Action, Completion};

pub const GRID_NAV_GROUP: &str = "grid_nav";

#[derive(GodotClass)]
#[class(base = Node, init)]
pub struct GridNav {
    base: Base<Node>,

    #[export] layer: Option<Gd<TileMapLayer>>,
    astar: Option<Gd<AStarGrid2D>>
}

#[godot_api]
impl INode for GridNav {
    fn ready(&mut self) {
        self.base_mut().add_to_group(GRID_NAV_GROUP);
        self.base_mut().call_deferred("rebuild", &[]);
    }
}

#[godot_api]
impl GridNav {
    #[func]
    pub fn rebuild(&mut self) {
        let Some(layer) = self.layer.clone() else {
            godot_error!("No TileMapLayer assigned to GridNav");
            return;
        };

        let region = layer.get_used_rect();
        
        let mut astar = AStarGrid2D::new_gd();
        astar.set_region(region);
        astar.set_diagonal_mode(DiagonalMode::NEVER);
        astar.update();
 
        for x in region.position.x..region.end().x {
            for y in region.position.y..region.end().y {
                let cell = Vector2i::new(x, y);
                let solid = match layer.get_cell_tile_data(cell) {
                    None => false,
                    Some(data) => data
                        .get_custom_data("solid")
                        .try_to::<bool>()
                        .unwrap_or(false)
                };
                if solid {
                    astar.set_point_solid(cell);
                }
            }
        }
        self.astar = Some(astar);
    }

    pub fn find_path(&self, from: Vector2i, to: Vector2i) -> Option<Vec<Vector2i>> {
        let astar = self.astar.clone()?;

        if !astar.is_in_boundsv(from) || !astar.is_in_boundsv(to) || astar.is_point_solid(to) {
            return None;
        }

        let path = astar.get_id_path(from, to);
        (!path.is_empty()).then(|| path.iter_shared().collect())
    }

    pub fn cell_to_world(&self, cell: Vector2i) -> Vector2 {
        let layer = self.layer.as_ref().expect("GridNav: no layer");
        layer.to_global(layer.map_to_local(cell))
    }

    pub fn world_to_cell(&self, pos: Vector2) -> Vector2i {
        let layer = self.layer.as_ref().expect("GridNav: no layer");
        layer.local_to_map(layer.to_local(pos))
    }
}

pub struct MoveTo {
    pub target: Vector2i,
    pub seconds_per_tile: f64
}

impl Action for MoveTo {
    fn start(self: Box<Self>, host: Gd<Node>, done: Completion) {
        let Ok(mut body) = host.try_cast::<Node2D>() else {
            return done.failed("MoveTo: host is not a Node2D");
        };

        let Some(nav) = body
            .get_tree()
            .get_first_node_in_group(GRID_NAV_GROUP)
            .and_then(|n| n.try_cast::<GridNav>().ok())
        else {
            return done.failed("MoveTo: no GridNav in scene");
        };

        let waypoints: Vec<Vector2> = {
            let nav = nav.bind();
            let start = nav.world_to_cell(body.get_global_position());

            if start == self.target {
                return done.success_with(self.target);
            }

            let Some(path) = nav.find_path(start, self.target) else {
                return done.failed(format!("MoveTo: no path from {} to {}", start, self.target));
            };

            path.iter().skip(1).map(|&cell| nav.cell_to_world(cell)).collect()
        };

        godot_print!("Moving to {:?} with {} waypoints", self.target, waypoints.len());

        let mut tween = body.create_tween();
        for point in waypoints {
            tween.tween_property(
                &body, 
                "global_position",
                &point.to_variant(),
                self.seconds_per_tile
            );
        }

        let finished = tween.signals().finished().to_fallible_future();
        let target = self.target;

        godot::task::spawn(async move {
            match finished.await {
                Ok(()) => done.success_with(target),
                Err(_) => done.failed("MoveTo: movement interrupted")
            }
        });
    }
}

