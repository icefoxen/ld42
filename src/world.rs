//! This file defines the `World`,
//! as well as some handy utility methods and structs.
//! The `World` contains shared state that will be available
//! to every `Scene`: specs objects, input state, asset cache.

use ggez;
use ggez_goodies::input as ginput;
use ncollide2d as nc;
use specs;

use warmy;

use std::path;

use components::*;
use input;
use util::*;

pub struct World {
    pub assets: warmy::Store<ggez::Context>,
    pub input: input::InputState,
    pub specs_world: specs::World,
    pub quit: bool,
}

impl World {
    fn register_components(&mut self) {
        self.specs_world.register::<Collider>();
        self.specs_world.register::<Motion>();
        self.specs_world.register::<Mass>();
        self.specs_world.register::<Player>();
        self.specs_world.register::<Sprite>();
        self.specs_world.register::<Mesh>();
        self.specs_world.register::<Gravity>();
        self.specs_world.register::<Obstacle>();
    }

    pub fn new(ctx: &mut ggez::Context, resource_dir: Option<path::PathBuf>) -> Self {
        // We to bridge the gap between ggez and warmy path
        // handling here; ggez assumes its own absolute paths, warmy
        // assumes system-absolute paths; so, we make warmy look in
        // the specified resource dir (normally
        // $CARGO_MANIFEST_DIR/resources) or the ggez default resource
        // dir.
        let resource_pathbuf: path::PathBuf = match resource_dir {
            Some(s) => s,
            None => ctx.filesystem.get_resources_dir().to_owned(),
        };
        info!("Setting up resource path: {:?}", resource_pathbuf);
        let opt = warmy::StoreOpt::default().set_root(resource_pathbuf);
        let store = warmy::Store::new(opt)
            .expect("Could not create asset store?  Does the directory exist?");

        let mut w = specs::World::new();
        let collide_world: CollisionWorld = nc::world::CollisionWorld::new(0.02);
        w.add_resource(collide_world);

        let mut the_world = Self {
            assets: store,
            input: ginput::InputState::new(),
            specs_world: w,
            quit: false,
        };

        the_world.register_components();

        the_world
    }
}
