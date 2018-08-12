use ggez;
use ggez::graphics;
use ggez_goodies::scene;
use input;
use scenes::*;
use world::World;


pub struct MenuScene {
    done: bool,
}

impl MenuScene {
    pub fn new() -> Self {
        Self {
            done: false,
        }
    }
}


impl scene::Scene<World, input::InputEvent> for MenuScene {
    fn update(&mut self, gameworld: &mut World) -> FSceneSwitch {
        if self.done {
            // See https://github.com/ggez/ggez-goodies/issues/11
            // We work around that...
            scene::SceneSwitch::Pop
        } else {
            scene::SceneSwitch::None
        }
    }

    fn draw(&mut self, _gameworld: &mut World, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        let t =
            ggez::graphics::TextCached::new(r#"
Running In To Space



Not actually done, but close!  You are an astronaut
trapped on a remote planet and need to run your way into orbit!



Directions: You will start running, just press Z to jump over obstacles.
Escape key quits.



Press Z to continue!
"#)?;

        t.queue(ctx, graphics::Point2::new(200.0, 100.0), Some(graphics::WHITE));

        graphics::TextCached::draw_queued(ctx, graphics::DrawParam::default())?;
        Ok(())
    }

    fn name(&self) -> &str {
        "MenuScene"
    }

    fn input(&mut self, gameworld: &mut World, _ev: input::InputEvent, _started: bool) {
        if gameworld.input.get_button_pressed(input::Button::Jump) {
            self.done = true;
        }
    }
}