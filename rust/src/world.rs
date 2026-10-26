// use godot::global::MouseButton;
// use godot::{classes::{FastNoiseLite, InputEvent, InputEventMouseButton, TileMapLayer, fast_noise_lite::{FractalType, NoiseType}}, prelude::*};


// const SOURCE_ID: i32 = 0;
// const GROUND: Vector2i = Vector2i::new(2, 6);

// #[derive(GodotClass)]
// #[class(base = Node2D)]
// pub struct Map {
//     #[export] width: i32,
//     #[export] height: i32,

//     #[export] radius_x: f32,
//     #[export] radius_y: f32,

//     #[export] noise_strength: f32,
//     #[export] world_seed: i32,

//     #[export] terrain_set: i32,
//     #[export] terrain: i32,

//     base: Base<Node2D>
// }

// #[godot_api]
// impl INode2D for Map {
//     fn init(base: Base<Node2D>) -> Self {
//         Self {
//             width: 120,
//             height: 190,
//             radius_x: 50.0,
//             radius_y: 32.0,
//             noise_strength: 0.4,
//             world_seed: 1234,
//             terrain_set: 0,
//             terrain: 0,
//             base
//         }
//     }

//     fn ready(&mut self) {
//         self.generate();
//     }
// }

// impl Map {
//     pub fn generate(&mut self) {
//         let (w, h) = (self.width, self.height);

//         let mut noise = FastNoiseLite::new_gd();
//         noise.set_seed(self.world_seed);
//         noise.set_noise_type(NoiseType::SIMPLEX_SMOOTH);
//         noise.set_fractal_type(FractalType::FBM);
//         noise.set_fractal_octaves(4);
//         noise.set_frequency(0.04);

//         let cx = w as f32 / 2.0;
//         let cy = h as f32 / 2.0;
//         let mut land = vec![vec![false; h as usize]; w as usize];
    
//         for x in 0..w {
//             for y in 0..h {
//                 let dx = (x as f32 - cx) / self.radius_x;
//                 let dy = (y as f32 - cy) / self.radius_y;
//                 let d = (dx * dx + dy * dy).sqrt();

//                 let n = noise.get_noise_2d(x as f32, y as f32);
//                 let value = (1.0 - d) + n * self.noise_strength;

//                 land[x as usize][y as usize] = value > 0.0;
//             }
//         }

//         for _ in 0..3 {
//             land = smooth(&land);
//         }

//         let mut cells: Array<Vector2i> = Array::new();
//         for x in 0..w {
//             for y in 0..h {
//                 if land[x as usize][y as usize] {
//                     cells.push(Vector2i::new(x, y));
//                 }
//             }
//         }

//         let world_bounds = Rect2i::new(
//             Vector2i::new(0, 0),
//             Vector2i::new(w, h)
//         );

//         let mut water = self.base().get_node_as::<TileMapLayer>("liquid");
//         water.clear();
//         fill_rect(&mut water, world_bounds, 0, Vector2i::new(4, 7));

//         let mut ground = self.base().get_node_as::<TileMapLayer>("ground");
//         ground.clear();
//         ground.set_cells_terrain_connect(&cells, self.terrain_set, self.terrain);

//     }
// }

// fn fill_rect(layer: &mut Gd<TileMapLayer>, rect: Rect2i, source_id: i32, atlas: Vector2i) {
//     let start = rect.position;
//     let end = rect.end();
//     for x in start.x..end.x {
//         for y in start.y..end.y {
//             layer
//                 .set_cell_ex(Vector2i::new(x, y))
//                 .source_id(source_id)
//                 .atlas_coords(atlas)
//                 .done();
//         }
//     }
// }


// #[derive(GodotClass)]
// #[class(init, base = Node2D)]
// pub struct BuildManager {
//     base: Base<Node2D>,

//     #[export]
//     #[init(val = 2)]
//     wall_source_id: i32,

//     #[export]
//     #[init(val = Vector2i::new(3, 6))]
//     wall_atlas_coords: Vector2i,
// }

// #[godot_api]
// impl INode2D for BuildManager {
//     fn unhandled_input(&mut self, event: Gd<InputEvent>) {
//         let Ok(mb) = event.try_cast::<InputEventMouseButton>() else {
//             return;
//         };

//         if !mb.is_pressed() || mb.get_button_index() != MouseButton::LEFT {
//             return;
//         }

//         let global_pos = self.base().get_global_mouse_position();
//         let mut structures = self.base().get_node_as::<TileMapLayer>("../structures");
//         let cell = structures.local_to_map(structures.to_local(global_pos));

//         let ground = self.base().get_node_as::<TileMapLayer>("../ground");
//         if !ground.get_used_rect().contains_point(cell) {
//             return;
//         }

//         structures
//             .set_cell_ex(cell)
//             .source_id(self.wall_source_id)
//             .atlas_coords(self.wall_atlas_coords)
//             .done();
//     }
// }

// fn smooth(grid: &[Vec<bool>]) -> Vec<Vec<bool>> {
//     let w = grid.len() as i32;
//     let h = grid[0].len() as i32;
//     let mut out = grid.to_vec();

//     for x in 0..w {
//         for y in 0..h {
//             let mut count = 0;
//             for ox in -1..=1 {
//                 for oy in -1..=1 {
//                     if ox == 0 && oy == 0 { continue; }
//                     let (nx, ny) = (x + ox, y + oy);
//                     if nx >= 0 && ny >= 0 && nx < w && ny < h
//                         && grid[nx as usize][ny as usize]
//                     {
//                         count += 1;
//                     }
//                 }
//             }
//             if count >= 5 {
//                 out[x as usize][y as usize] = true;
//             } else if count <= 3 {
//                 out[x as usize][y as usize] = false;
//             }
//         }
//     }
//     out
// }