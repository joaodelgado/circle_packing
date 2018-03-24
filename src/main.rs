extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use piston::window::WindowSettings;
use piston::event_loop::{EventSettings, Events};
use opengl_graphics::{GlGraphics, OpenGL};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use glutin_window::GlutinWindow as Window;
use rand::{thread_rng as random, Rng};

use graphics::types::Color;
use graphics::color::{BLACK, WHITE};

const WIDTH: u32 = 1280;
const HEIGTH: u32 = 720;

const CIRCLES_PER_SECOND: u32 = 100;
const SECONDS_PER_CIRCLE: f64 = 1.0 / CIRCLES_PER_SECOND as f64;

const CIRCLE_SPACING: u32 = 30;
const CIRCLE_BORDER: f64 = 1.0;
const CIRCLE_EXPANSION_RATE: f64 = 50.0;

#[derive(PartialEq, Clone)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn dst(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

#[derive(PartialEq, Clone)]
struct Circle {
    center: Point,
    color: Color,
    radius: f64,
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

    fn render(&mut self, args: &RenderArgs, gl: &mut GlGraphics) {
        use graphics::ellipse::{circle, Ellipse};

        gl.draw(args.viewport(), |c, gl| {
            Ellipse::new_border(self.color, CIRCLE_BORDER).draw(
                circle(self.center.x, self.center.y, self.radius),
                &c.draw_state,
                c.transform,
                gl,
            );
        });
    }

    fn is_colliding(&self, other: &Circle) -> bool {
        let minimum_space = self.radius + CIRCLE_BORDER + CIRCLE_BORDER + other.radius;
        self.center.dst(&other.center) < minimum_space
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn is_edge_colliding(&mut self) -> bool {
        let space = self.radius + CIRCLE_BORDER;
        self.center.x + space >= WIDTH as f64
            || self.center.x - space <= 0.0
            || self.center.y + space >= HEIGTH as f64
            || self.center.y - space <= 0.0
    }

    fn expand(&mut self, args: &UpdateArgs) {
        if self.expanding {
            self.radius += CIRCLE_EXPANSION_RATE * args.dt;
        }
    }
}

struct App {
    circles: Vec<Circle>,
    available_spaces: Vec<Point>,
    dt_to_next_circle: f64,
}

impl App {
    fn new() -> App {
        let mut available_spaces = vec![];
        for x in 0..(WIDTH / CIRCLE_SPACING) {
            for y in 0..(HEIGTH / CIRCLE_SPACING) {
                available_spaces.push(Point {
                    x: (x * CIRCLE_SPACING) as f64,
                    y: (y * CIRCLE_SPACING) as f64,
                });
            }
        }
        random().shuffle(&mut available_spaces);

        App {
            circles: vec![],
            available_spaces: available_spaces,
            dt_to_next_circle: 0.0,
        }
    }

    fn render(&mut self, args: &RenderArgs, gl: &mut GlGraphics) {
        use graphics::clear;

        clear(BLACK, gl);

        self.circles
            .iter_mut()
            .for_each(|circle| circle.render(args, gl));
    }

    fn update(&mut self, args: &UpdateArgs) {
        if self.dt_to_next_circle <= 0.0 {
            self.create_circle();
            self.dt_to_next_circle = SECONDS_PER_CIRCLE;
        }
        self.dt_to_next_circle -= args.dt;

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
            circle.expand(args);
        }
    }

    fn create_circle(&mut self) {
        if self.available_spaces.is_empty() {
            return;
        }

        loop {
            if let Some(center) = self.available_spaces.pop() {
                let new = Circle::new_at(center);
                if self.circles
                    .iter()
                    .all(|existing| !new.is_colliding(existing))
                {
                    self.circles.push(new);
                    println!("Spaces to fill: {}", self.available_spaces.len());
                    return;
                }
            } else {
                println!("Done!");
                return;
            }
        }
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    let mut window: Window = WindowSettings::new("Space filling circles", [WIDTH, HEIGTH])
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .expect("Error creating window");

    let mut gl = GlGraphics::new(opengl);
    let mut app = App::new();

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r, &mut gl);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }
    }
}
