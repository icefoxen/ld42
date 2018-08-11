use ggez;
use ggez::graphics;
use ggez_goodies::scene;
use ncollide2d as nc;
use nalgebra as na;
use specs::{self, Join};
use warmy;

use input;
use components::*;
use error::Err;
use scenes::*;
use systems::*;
use resources;
use world::World;
use util::*;

pub struct LevelScene {
    done: bool,
    kiwi: warmy::Res<resources::Image>,
    dispatcher: specs::Dispatcher<'static, 'static>,
    collide_world: nc::world::CollisionWorld<f32, ()>,
}

impl LevelScene {
    pub fn new(ctx: &mut ggez::Context, world: &mut World) -> Result<Self, Err> {
        let done = false;
        let kiwi = world
            .assets
            .get::<_, resources::Image>(&warmy::FSKey::new("/images/kiwi.png"), ctx)
            .unwrap();


        // Make the player.
        world
            .specs_world
            .create_entity()
            .with(Position(Point2::new(10.0, 10.0)))
            .with(Motion {
                velocity: Vector2::new(1.0, 0.0),
                acceleration: Vector2::new(0.0, 0.0),
            })
            .with(Mass {})
            .with(Sprite {})
            .build();

        let ball = nc::shape::Ball::new(10.0);

        let dispatcher = Self::register_systems();
        let mut terrain_collide_group = nc::world::CollisionGroups::new();
        terrain_collide_group.set_membership(&[1]);
        let query_type = nc::world::GeometricQueryType::Contacts(0.0, 0.0);

        let mut collide_world = nc::world::CollisionWorld::new(0.02);
        let planet_handle = collide_world.add(
            na::Isometry2::new(na::zero(), na::zero()),
            nc::shape::ShapeHandle::new(ball.clone()),
            terrain_collide_group,
            query_type,
            ()
        );

        let planet_collider = Collider {
            object: planet_handle,
        };

        // Make the world object thingy
        world.specs_world
            .create_entity()
            .with(Position(Point2::new(0.0, 0.0)))
            .with(Mesh {
                mesh: graphics::MeshBuilder::default()
                    .circle(graphics::DrawMode::Fill,
                        graphics::Point2::new(0.0, 0.0), 10.0, 1.0)
                    .build(ctx)?
            })
            .with(planet_collider)
            .build();

        Ok(LevelScene {
            done,
            kiwi,
            dispatcher,
            collide_world,
        })
    }



    fn register_systems() -> specs::Dispatcher<'static, 'static> {
        let gravity = GravitySystem {
            position: Point2::new(0.0, 0.0),
            force: 1.0,
        };
        specs::DispatcherBuilder::new()
            .add(MovementSystem, "sys_movement", &[])
            .add(gravity, "sys_gravity", &[])
            // .add(DebugPrinterSystem {}, "sys_debugprint", &[])
            .build()
    }
}

impl scene::Scene<World, input::InputEvent> for LevelScene {
    fn update(&mut self, gameworld: &mut World) -> FSceneSwitch {
        self.dispatcher
            .dispatch(&mut gameworld.specs_world.res);

        for e in self.collide_world.contact_events() {
            println!("{:?}", e);
        }

        if self.done {
            scene::SceneSwitch::Pop
        } else {
            scene::SceneSwitch::None
        }
    }

    fn draw(&mut self, gameworld: &mut World, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        let pos = gameworld.specs_world.read::<Position>();
        let sprite = gameworld.specs_world.read::<Sprite>();
        let mesh = gameworld.specs_world.read::<Mesh>();
        for (p, _) in (&pos, &sprite).join() {
            graphics::draw(ctx, &(self.kiwi.borrow().0), graphics::Point2::new(p.0.x, p.0.y), 0.0)?;
        }

        for (p, mesh) in (&pos, &mesh).join() {
            graphics::draw(ctx, &mesh.mesh, graphics::Point2::new(p.0.x, p.0.y), 0.0)?;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "LevelScene"
    }

    fn input(&mut self, gameworld: &mut World, ev: input::InputEvent, _started: bool) {
        debug!("Input: {:?}", ev);
        if gameworld.input.get_button_pressed(input::Button::Menu) {
            gameworld.quit = true;
        }
    }
}
