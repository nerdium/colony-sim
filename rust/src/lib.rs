use godot::prelude::*;

pub mod action;
pub mod camera;
pub mod nav;
pub mod npc;
pub mod task;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}