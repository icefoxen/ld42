use ggez;
use ggez::graphics;
use ggez_goodies::scene;
use ncollide2d as nc;
use nalgebra as na;
use specs::{self, Join, Builder};
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
}

impl LevelScene {
    pub fn new(ctx: &mut ggez::Context, world: &mut World) -> Result<Self, Err> {
        let done = false;
        let kiwi = world
            .assets
            .get::<_, resources::Image>(&warmy::FSKey::new("/images/kiwi.png"), ctx)
            .unwrap();


        let ball = nc::shape::Ball::new(10.0);
        let dispatcher = Self::register_systems();

        // Planet collision info
        let mut terrain_collide_group = nc::world::CollisionGroups::new();
        terrain_collide_group.set_membership(&[1]);
        let query_type = nc::world::GeometricQueryType::Contacts(0.0, 0.0);

        let planet_collider = {
            let mut collide_world = world.specs_world.write_resource::<CollisionWorld>();
            let planet_handle = collide_world.add(
                na::Isometry2::new(na::zero(), na::zero()),
                nc::shape::ShapeHandle::new(ball.clone()),
                terrain_collide_group,
                query_type,
                ()
            );

            Collider {
                object_handle: planet_handle,
            }
        };

        // Make the world object thingy
        world.specs_world
            .create_entity()
            .with(Mesh {
                mesh: graphics::MeshBuilder::default()
                    .circle(graphics::DrawMode::Fill,
                        graphics::Point2::new(0.0, 0.0), 10.0, 1.0)
                    .build(ctx)?
            })
            .with(planet_collider)
            .build();

        // Player collision info
        let mut player_collide_group = nc::world::CollisionGroups::new();
        player_collide_group.set_membership(&[2]);
        let query_type = nc::world::GeometricQueryType::Contacts(0.0, 0.0);

        let player_collider = {
            let mut collide_world = world.specs_world.write_resource::<CollisionWorld>();
            let player_handle = collide_world.add(
                na::Isometry2::new(na::Vector2::new(10.0, 10.0), na::zero()),
                nc::shape::ShapeHandle::new(ball.clone()),
                player_collide_group,
                query_type,
                ()
            );

            Collider {
                object_handle: player_handle,
            }
        };


        // Make the player.
        world
            .specs_world
            .create_entity()
            .with(Position {
                position: Point2::new(10.0, 10.0),
                orientation: 0.0,
            })
            .with(Motion {
                velocity: Vector2::new(1.0, 0.0),
                acceleration: Vector2::new(0.0, 0.0),
            })
            .with(Mass {})
            .with(Sprite {})
            .with(player_collider)
            .build();



        Ok(LevelScene {
            done,
            kiwi,
            dispatcher,
        })
    }



    fn register_systems() -> specs::Dispatcher<'static, 'static> {
        let gravity = GravitySystem {
            position: Point2::new(0.0, 0.0),
            force: 1.0,
        };
        specs::DispatcherBuilder::new()
            // .with(MovementSystem, "sys_movement", &[])
            .with(NCollideMotionSystem {}, "sys_movement", &[])
            .with(gravity, "sys_gravity", &["sys_movement"])
            // .with(DebugPrinterSystem {}, "sys_debugprint", &[])
            .build()
    }
}

impl scene::Scene<World, input::InputEvent> for LevelScene {
    fn update(&mut self, gameworld: &mut World) -> FSceneSwitch {
        self.dispatcher
            .dispatch(&mut gameworld.specs_world.res);

        let collide_world = gameworld.specs_world.read_resource::<CollisionWorld>();
        for e in collide_world.contact_events() {
            println!("{:?}", e);
        }

        if self.done {
            scene::SceneSwitch::Pop
        } else {
            scene::SceneSwitch::None
        }
    }

    fn draw(&mut self, gameworld: &mut World, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        let sprite = gameworld.specs_world.read_storage::<Sprite>();
        let mesh = gameworld.specs_world.read_storage::<Mesh>();
        let collider = gameworld.specs_world.read_storage::<Collider>();
        let ncollide_world = gameworld.specs_world.read_resource::<CollisionWorld>();
        for (c, _) in (&collider, &sprite).join() {
            let collision_object = ncollide_world.collision_object(c.object_handle)
                .expect("Invalid collision object; was it removed from ncollide but not specs?");
            let isometry = collision_object.position();
            let annoying_new_pos = graphics::Point2::new(isometry.translation.vector.x, isometry.translation.vector.y);
            let annoying_new_angle = isometry.rotation.angle();
            graphics::draw_ex(ctx, &(self.kiwi.borrow().0),
                graphics::DrawParam {
                    dest: annoying_new_pos,
                    rotation: annoying_new_angle,
                    .. graphics::DrawParam::default()
                })?;
        }

        for (c, mesh) in (&collider, &mesh).join() {
            let collision_object = ncollide_world.collision_object(c.object_handle)
                .expect("Invalid collision object; was it removed from ncollide but not specs?");
            let isometry = collision_object.position();
            let annoying_new_pos = graphics::Point2::new(isometry.translation.vector.x, isometry.translation.vector.y);
            let annoying_new_angle = isometry.rotation.angle();
            graphics::draw_ex(ctx, &mesh.mesh,
                graphics::DrawParam {
                    dest: annoying_new_pos,
                    rotation: annoying_new_angle,
                    .. graphics::DrawParam::default()
                })?;
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
