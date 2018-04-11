extern crate ggez;
extern crate rand;

use rand::{thread_rng as random, Rng};

use ggez::graphics::Color;
use ggez::graphics::{BLACK, WHITE};
use ggez::*;

type Point = nalgebra::geometry::Point2<f32>;

const WIDTH: u32 = 1280;
const HEIGTH: u32 = 720;

const CIRCLES_PER_SECOND: u32 = 60;
const SECONDS_PER_CIRCLE: f64 = 1.0 / CIRCLES_PER_SECOND as f64;

const CIRCLE_SPACING: u32 = 15;
const CIRCLE_BORDER: f32 = 1.0;
const CIRCLE_EXPANSION_RATE: f32 = 75.0;

#[derive(PartialEq, Clone)]
struct Circle {
    center: Point,
    color: Color,
    radius: f32,
    expanding: bool,
}

impl Circle {
    fn new_at(center: Point) -> Circle {
        Circle {
            center: center,
            color: WHITE,
            radius: 0.0,
            expanding: true,
        }
    }

    fn render(&mut self, ctx: &mut Context) -> GameResult<()> {
        use graphics::DrawMode;
        graphics::circle(
            ctx,
            DrawMode::Line(CIRCLE_BORDER),
            self.center,
            self.radius,
            0.1,
        )?;
        Ok(())
    }

    fn is_colliding(&self, other: &Circle) -> bool {
        use nalgebra::distance;
        let minimum_space = self.radius + CIRCLE_BORDER + CIRCLE_BORDER + other.radius;
        distance(&self.center, &other.center) < minimum_space
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn is_edge_colliding(&mut self) -> bool {
        let space = self.radius + CIRCLE_BORDER;
        self.center.x + space >= WIDTH as f32
            || self.center.x - space <= 0.0
            || self.center.y + space >= HEIGTH as f32
            || self.center.y - space <= 0.0
    }

    fn expand(&mut self, ctx: &Context) {
        if self.expanding {
            self.radius +=
                CIRCLE_EXPANSION_RATE * timer::duration_to_f64(timer::get_delta(ctx)) as f32;
        }
    }
}

struct MainState {
    circles: Vec<Circle>,
    available_spaces: Vec<Point>,
    dt_to_next_circle: f64,
}

impl MainState {
    fn new(ctx: &mut Context) -> MainState {
        graphics::set_background_color(ctx, BLACK);

        let mut available_spaces = vec![];
        for x in 0..(WIDTH / CIRCLE_SPACING) {
            for y in 0..(HEIGTH / CIRCLE_SPACING) {
                available_spaces.push(Point::new(
                    (x * CIRCLE_SPACING) as f32,
                    (y * CIRCLE_SPACING) as f32,
                ));
            }
        }
        random().shuffle(&mut available_spaces);

        MainState {
            circles: vec![],
            available_spaces: available_spaces,
            dt_to_next_circle: 0.0,
        }
    }

    fn create_circle(&mut self, ctx: &mut Context) {
        if self.available_spaces.is_empty() {
            return;
        }

        while timer::check_update_time(ctx, 60) {
            if let Some(center) = self.available_spaces.pop() {
                println!("Spaces to fill: {}", self.available_spaces.len());
                let new = Circle::new_at(center);
                if self.circles
                    .iter()
                    .all(|existing| !new.is_colliding(existing))
                {
                    self.circles.push(new);
                    return;
                }
            } else {
                return;
            }
        }
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if self.dt_to_next_circle <= 0.0 {
            self.create_circle(ctx);
            self.dt_to_next_circle = SECONDS_PER_CIRCLE as f64;
        }
        self.dt_to_next_circle -= timer::duration_to_f64(timer::get_delta(ctx));

        let other_circles = self.circles.clone();
        for c1 in self.circles.iter_mut().filter(|c| c.expanding) {
            if c1.is_edge_colliding() {
                c1.expanding = false;
                continue;
            }

            for c2 in other_circles.iter() {
                if c2 == c1 {
                    continue;
                }

                if c1.is_colliding(&c2) {
                    c1.expanding = false;
                    break;
                }
            }
        }
        for circle in self.circles.iter_mut().filter(|c| c.expanding) {
            circle.expand(ctx);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        for circle in &mut self.circles {
            circle.render(ctx)?;
        }

        graphics::present(ctx);
        timer::yield_now();
        Ok(())
    }
}

fn main() {
    let mut ctx = ContextBuilder::new("Space filling circles", "João Delgado")
        .window_setup(conf::WindowSetup::default().title("Space filling circles"))
        .window_mode(conf::WindowMode::default().dimensions(WIDTH, HEIGTH))
        .build()
        .expect("Error building context");
    let mut state = MainState::new(&mut ctx);
    if let Err(e) = event::run(&mut ctx, &mut state) {
        println!("[ERROR] {}", e);
    }
}
