use ggez;
use ggez::graphics;
use ggez_goodies::scene;
use nalgebra as na;
use ncollide2d as nc;
use rand;
use specs::{self, Builder, Join};
use warmy;

use std::f32;

use components::*;
use error::Err;
use input;
use resources;
use scenes::*;
use systems::*;
use util::*;
use world::World;

pub struct LevelScene {
    done: bool,
    sprite: warmy::Res<resources::Image>,
    dispatcher: specs::Dispatcher<'static, 'static>,
    player_entity: specs::Entity,
    planet_entity: specs::Entity,
    camera_focus: Point2,
    background_mesh: graphics::Mesh,
}

const CAMERA_WIDTH: f32 = 800.0;
const CAMERA_HEIGHT: f32 = 600.0;

impl LevelScene {
    pub fn new(ctx: &mut ggez::Context, world: &mut World) -> Result<Self, Err> {
        let done = false;
        let sprite = world
            .assets
            .get::<_, resources::Image>(&warmy::FSKey::new("/images/kiwi.png"), ctx)
            .unwrap();

        let dispatcher = Self::register_systems();

        let planet_radius = 2000.0;
        let planet_entity = Self::create_planet(ctx, world, planet_radius)?;
        let player_entity = Self::create_player(ctx, world, planet_radius)?;
        for i in 0..10 {
            let _ = Self::create_obstacle(ctx, world, planet_radius, f32::consts::PI + (i as f32 / 5.0))?;
        }

        let background_mesh = Self::create_background_mesh(ctx)?;

        Ok(LevelScene {
            done,
            sprite,
            dispatcher,
            player_entity,
            planet_entity,
            background_mesh,
            camera_focus: na::origin(),
        })
    }

    fn register_systems() -> specs::Dispatcher<'static, 'static> {
        let gravity = GravitySystem {
        };
        specs::DispatcherBuilder::new()
            .with(gravity, "sys_gravity", &[])
            // .with(NCollideMotionSystem {}, "sys_motion", &[])
            // .with(DebugPrinterSystem {}, "sys_debugprint", &[])
            .build()
    }

    fn create_background_mesh(ctx: &mut ggez::Context) -> Result<graphics::Mesh, Err> {
        let num_stars = 5000;
        let star_max_bounds = 5000.0;
        let mut mb = graphics::MeshBuilder::new();
        for _ in 0..num_stars {
            let x = rand::random::<f32>() * star_max_bounds - (star_max_bounds / 2.0);
            let y = rand::random::<f32>() * star_max_bounds - (star_max_bounds / 2.0);
            mb.circle(
                graphics::DrawMode::Fill,
                graphics::Point2::new(x, y),
                2.0,
                2.0);
        }
        mb.build(ctx)
            .map_err(Err::from)
    }

    fn create_player(ctx: &mut ggez::Context, world: &mut World, planet_radius: f32) -> Result<specs::Entity, Err> {
        let player_halfwidth = 10.0;
        let player_halfheight = 20.0;
        let run_acceleration = 0.01;
        let player_offset = planet_radius + player_halfheight*3.0;
        // Make the player entity
        let entity = world
            .specs_world
            .create_entity()
            .with(Player {
                on_ground: false,
                jumping: false,
                velocity: 0.0,
                run_acceleration,
            })
            .with(Motion {
                velocity: Vector2::new(1.5, 0.0),
                acceleration: Vector2::new(0.0, 0.0),
            })
            .with(Mass {})
            .with(Mesh {
                mesh: graphics::MeshBuilder::default()
                    .polygon(
                        graphics::DrawMode::Line(2.0),
                        &[
                            graphics::Point2::new(-player_halfwidth, -player_halfheight),
                            graphics::Point2::new(-player_halfwidth, player_halfheight),
                            graphics::Point2::new(player_halfwidth, player_halfheight),
                            graphics::Point2::new(player_halfwidth, -player_halfheight),
                        ],
                    )
                    .build(ctx)?,
            })
            .build();

        // Player collision info
        let shape = nc::shape::Cuboid::new(Vector2::new(player_halfwidth, player_halfheight));
        let mut player_collide_group = nc::world::CollisionGroups::new();
        player_collide_group.set_membership(&[2]);
        let query_type = nc::world::GeometricQueryType::Contacts(0.0, 0.0);

        let player_collider = {
            let mut collide_world = world.specs_world.write_resource::<CollisionWorld>();
            let player_handle = collide_world.add(
                na::Isometry2::new(na::Vector2::new(0.0, -player_offset), na::zero()),
                nc::shape::ShapeHandle::new(shape.clone()),
                player_collide_group,
                query_type,
                entity,
            );

            Collider {
                object_handle: player_handle,
            }
        };
        // Insert the collider.
        world.specs_world.write_storage::<Collider>().insert(entity, player_collider)?;
        Ok(entity)
    }

    fn create_planet(ctx: &mut ggez::Context, world: &mut World, planet_radius: f32) -> Result<specs::Entity, Err> {
        let gravity = 200.0;
        // Make the world entity
        let entity = world
            .specs_world
            .create_entity()
            .with(Mesh {
                mesh: graphics::MeshBuilder::default()
                    .circle(
                        graphics::DrawMode::Fill,
                        graphics::Point2::new(0.0, 0.0),
                        planet_radius,
                        0.1,
                    )
                    .build(ctx)?,
            })
            .with(Gravity {
                    force: gravity,
                }
            )
            .build();

        // Planet collision info
        let ball = nc::shape::Ball::new(planet_radius);
        let mut terrain_collide_group = nc::world::CollisionGroups::new();
        terrain_collide_group.set_membership(&[1]);
        let query_type = nc::world::GeometricQueryType::Contacts(0.0, 0.0);

        let planet_collider = {
            let mut collide_world = world.specs_world.write_resource::<CollisionWorld>();
            let planet_handle = collide_world.add(
                na::Isometry2::new(Vector2::new(0.0, 0.0), na::zero()),
                nc::shape::ShapeHandle::new(ball.clone()),
                terrain_collide_group,
                query_type,
                entity,
            );

            Collider {
                object_handle: planet_handle,
            }
        };
        // Insert the collider.
        world.specs_world.write_storage::<Collider>().insert(entity, planet_collider)?;
        Ok(entity)
    }


    /// Creates an obstacle on the planet at the given angle.
    /// Assumes the planet is at 0,0 I guess
    fn create_obstacle(ctx: &mut ggez::Context, world: &mut World, planet_radius: f32, angle: f32) -> Result<specs::Entity, Err> {
        let obstacle_halfwidth = 10.0;
        let obstacle_offset = planet_radius + obstacle_halfwidth;
        // Make the player entity
        let entity = world
            .specs_world
            .create_entity()
            .with(Obstacle {})
            .with(Mesh {
                mesh: graphics::MeshBuilder::default()
                    .polygon(
                        graphics::DrawMode::Line(2.0),
                        &[
                            graphics::Point2::new(-obstacle_halfwidth, -obstacle_halfwidth),
                            graphics::Point2::new(-obstacle_halfwidth, obstacle_halfwidth),
                            graphics::Point2::new(obstacle_halfwidth, obstacle_halfwidth),
                            graphics::Point2::new(obstacle_halfwidth, -obstacle_halfwidth),
                        ],
                    )
                    .build(ctx)?,
            })
            .build();

        // collision info
        let shape = nc::shape::Cuboid::new(Vector2::new(obstacle_halfwidth, obstacle_halfwidth));
        // TODO: Wait do we create multiple groups here?  I think so...
        // Also we gotta make sure we keep the things straight.
        // TODO: Figure out membership; must collide with player but not
        // the planet.
        let mut obstacle_collide_group = nc::world::CollisionGroups::new();
        obstacle_collide_group.set_membership(&[3]);
        let query_type = nc::world::GeometricQueryType::Contacts(0.0, 0.0);

        let obstacle_collider = {
            let mut collide_world = world.specs_world.write_resource::<CollisionWorld>();
            // sigh
            let x = f32::cos(angle) * obstacle_offset;
            let y = f32::sin(angle) * obstacle_offset;
            let handle = collide_world.add(
                na::Isometry2::new(
                    na::Vector2::new(x,y),
                    angle,
                ),
                nc::shape::ShapeHandle::new(shape.clone()),
                obstacle_collide_group,
                query_type,
                entity,
            );

            Collider {
                object_handle: handle,
            }
        };
        // Insert the collider.
        world.specs_world.write_storage::<Collider>().insert(entity, obstacle_collider)?;
        Ok(entity)

    }

    fn handle_contact_events(&mut self, gameworld: &mut World) {
        let mut collide_world = gameworld.specs_world.write_resource::<CollisionWorld>();
        collide_world.update();
        let mut player_storage = gameworld.specs_world.write_storage::<Player>();

        // Save and reuse the same vec each run of the loop so we only allocate once.
        let contacts_list = &mut Vec::new();
        for e in collide_world.contact_events() {
            contacts_list.clear();
            match e {
                nc::events::ContactEvent::Started(cobj_handle1, cobj_handle2) => {
                    // It's apparently possible for the collision pair to have
                    // no contacts...
                    // Possibly if one object is entirely inside another?
                    if let Some(pair) = (&*collide_world).contact_pair(*cobj_handle1, *cobj_handle2) {
                        pair.contacts(contacts_list);
                        let cobj1 = collide_world.collision_object(*cobj_handle1)
                            .expect("Invalid collision object handle?");
                        let cobj2 = collide_world.collision_object(*cobj_handle2)
                            .expect("Invalid collision object handle?");

                        // Get the entities out of the collision data
                        let e1 = cobj1.data();
                        let e2 = cobj2.data();
                        for e in &[e1, e2] {
                            if let Some(player) = player_storage.get_mut(**e) {
                                player.on_ground = true;
                            }
                        }
                    }
                }
                nc::events::ContactEvent::Stopped(cobj_handle1, cobj_handle2) => {
                    if let Some(pair) = (&*collide_world).contact_pair(*cobj_handle1, *cobj_handle2) {
                        pair.contacts(contacts_list);
                        let cobj1 = collide_world.collision_object(*cobj_handle1)
                            .expect("Invalid collision object handle?");
                        let cobj2 = collide_world.collision_object(*cobj_handle2)
                            .expect("Invalid collision object handle?");

                        // Get the entities out of the collision data
                        let e1 = cobj1.data();
                        let e2 = cobj2.data();
                        for e in &[e1, e2] {
                            if let Some(player) = player_storage.get_mut(**e) {
                                player.on_ground = false;
                            }
                        }
                    }
                }
            }
        }
    }



    /// This is really hard to express as a specs System so we roll our own.
    fn run_player_motion(&mut self, world: &mut World) {
        if let Some(player) = world.specs_world.write_storage::<Player>().get_mut(self.player_entity) {
            let mut colliders = world.specs_world.write_storage::<Collider>();
            let mut motions = world.specs_world.write_storage::<Motion>();
            let mut ncollide_world = world.specs_world.write_resource::<CollisionWorld>();

            let player_motion = motions.get_mut(self.player_entity)
                .expect("Player w/o motion?");
            let player_collider = colliders.get(self.player_entity)
                .expect("Player w/o motion?");
            let (player_position, _player_rotation) = collision_object_position(&*ncollide_world, &player_collider);
            let planet_collider = colliders.get(self.planet_entity)
                .expect("Planet w/o collider?");
            let (planet_position, _planet_rotation) = collision_object_position(&*ncollide_world, planet_collider);

            let offset = player_position - planet_position;
            let normal = offset / na::norm(&offset);
            if player.on_ground {
                // We only want to zero the Y component
                // of the velocity... that is, the portion
                // towards the planet.
                // So we take the projection of velocity onto
                // the toward-the-planet offset vector.
                let projection = na::dot(&player_motion.velocity, &(offset / na::norm(&offset)));
                // debug!("Projection is {:?}, offset is {:?}, velocity is {}", projection, offset, player_motion.velocity);

                player_motion.velocity -= na::normalize(&offset) * projection;

                player_motion.acceleration = na::zero();
                // Jump
                if player.jumping {
                    player_motion.acceleration += normal;
                    player.on_ground = false;
                }

                // Walk
                let rot = na::Rotation2::new(f32::consts::PI / 2.0);
                let run_speed = rot * (normal * player.velocity);
                player_motion.acceleration += run_speed * player.run_acceleration;
            }
            player.velocity += player.run_acceleration;
            player_motion.velocity += player_motion.acceleration;
            player_motion.acceleration = na::zero();

            // Rotate to stand upright on planet.
            let player_angle = f32::atan2(offset.x, -offset.y);

            let new_position = {
                let collision_obj = ncollide_world
                    .collision_object(player_collider.object_handle)
                    .expect(
                        "Invalid collision object; was it removed from ncollide but not specs?",
                    );
                let mut new_position = collision_obj.position().clone();
                new_position.append_translation_mut(&na::Translation::from_vector(player_motion.velocity));
                new_position.rotation = na::UnitComplex::from_angle(player_angle);
                new_position
            };
            ncollide_world.set_position(player_collider.object_handle, new_position);
            self.camera_focus = Point2::new(new_position.translation.vector.x, new_position.translation.vector.y);
        }
    }
}

/// Takes a collision object handle and returns the location and orientation
/// of the object.
fn collision_object_position(
    ncollide_world: &CollisionWorld,
    collider: &Collider,
) -> (Point2, f32) {
    let collision_object = ncollide_world
        .collision_object(collider.object_handle)
        .expect("Invalid collision object; was it removed from ncollide but not specs?");
    let isometry = collision_object.position();
    let annoying_new_pos =
        Point2::new(isometry.translation.vector.x, isometry.translation.vector.y);
    let annoying_new_angle = isometry.rotation.angle();
    (annoying_new_pos, annoying_new_angle)
}



/// augh
///
/// Mainly used for drawing, so it returns ggez's Point type rather than ncollide's.
fn ggez_collision_object_position(
    ncollide_world: &CollisionWorld,
    collider: &Collider,
) -> (graphics::Point2, f32) {
    let (point, rotation) = collision_object_position(ncollide_world, collider);
    (graphics::Point2::new(point.x, point.y), rotation)
}


impl scene::Scene<World, input::InputEvent> for LevelScene {
    fn update(&mut self, gameworld: &mut World) -> FSceneSwitch {
        self.run_player_motion(gameworld);
        self.dispatcher.dispatch(&mut gameworld.specs_world.res);

        self.handle_contact_events(gameworld);
        if self.done {
            scene::SceneSwitch::Pop
        } else {
            scene::SceneSwitch::None
        }
    }

    fn draw(&mut self, gameworld: &mut World, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        // Focus view on player.
        let screen_rect = graphics::Rect {
            x: self.camera_focus.x - CAMERA_WIDTH/2.0,
            y: self.camera_focus.y - CAMERA_HEIGHT/2.0,
            w: CAMERA_WIDTH,
            h: CAMERA_HEIGHT,
        };
        graphics::set_screen_coordinates(ctx, screen_rect)?;

        // Draw background
        graphics::draw(ctx, &self.background_mesh, ggez::nalgebra::origin(), 0.0)?;

        let sprite = gameworld.specs_world.read_storage::<Sprite>();
        let mesh = gameworld.specs_world.read_storage::<Mesh>();
        let collider = gameworld.specs_world.read_storage::<Collider>();
        let ncollide_world = gameworld.specs_world.read_resource::<CollisionWorld>();
        for (c, _) in (&collider, &sprite).join() {
            let (pos, angle) = ggez_collision_object_position(&*ncollide_world, c);
            graphics::draw_ex(
                ctx,
                &(self.sprite.borrow().0),
                graphics::DrawParam {
                    dest: pos,
                    rotation: angle,
                    ..graphics::DrawParam::default()
                },
            )?;
        }

        for (c, mesh) in (&collider, &mesh).join() {
            let (pos, angle) = ggez_collision_object_position(&*ncollide_world, c);
            graphics::draw_ex(
                ctx,
                &mesh.mesh,
                graphics::DrawParam {
                    dest: pos,
                    rotation: angle,
                    ..graphics::DrawParam::default()
                },
            )?;
        }

        let player_storage = gameworld.specs_world.read_storage::<Player>();
        let player_component = player_storage.get(self.player_entity).expect("No player?");

        let text_point = graphics::Point2::new(10.0, 10.0);
        let velocity_point = graphics::Point2::new(10.0, 30.0);
        if player_component.on_ground {
            let t = ggez::graphics::TextCached::new("On ground")?;
            t.queue(ctx, text_point, None);
        } else {
            let t = ggez::graphics::TextCached::new("Not on ground")?;
            t.queue(ctx, text_point, None);
        }
        let t = ggez::graphics::TextCached::new(format!("Velocity: {}", player_component.velocity))?;
        t.queue(ctx, velocity_point, None);
        graphics::TextCached::draw_queued(ctx, graphics::DrawParam::default())?;
        Ok(())
    }

    fn name(&self) -> &str {
        "LevelScene"
    }

    fn input(&mut self, gameworld: &mut World, _ev: input::InputEvent, _started: bool) {
        if gameworld.input.get_button_pressed(input::Button::Menu) {
            gameworld.quit = true;
        }
        if let Some(player) = gameworld.specs_world.write_storage::<Player>().get_mut(self.player_entity) {
            player.jumping = gameworld.input.get_button_pressed(input::Button::Jump);
            // player.walk_direction = gameworld.input.get_axis(input::Axis::Horz);
        }
    }
}
