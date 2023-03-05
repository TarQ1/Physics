#![recursion_limit = "20000"]

use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    cmp::{self, max, min},
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::Mutex,
};

use generational_arena::Arena;
use raylib::prelude::*;

const HEIGHT: isize = 480;
const WIDTH: isize = 640;
const GRAVITY: f32 = 2.0;
const BALL_RADIUS: f32 = 10.0;
const FRICTION: f32 = 0.90;
const ACCELERATION: f32 = 0.5;
const MAX_SPEED: f32 = 20.0;
const BALL_MASS: f32 = 1.0;
const BALL_COEFF_RESTITUTION: f32 = 0.8;


fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Hello, World")
        .build();

    // limit the framerate to 60
    rl.set_target_fps(30);

    let mut arena: Arena<Ball> = Arena::new();

    let mut color_generator = ColorGenerator::new();

    let mut game_state = GameState {
        gravity: GRAVITY,
        moves: vec![],
        arena: arena,
        collisions: vec![],
        balls: vec![],
        color_generator: color_generator,
    };
    game_state.add_ball(
        (250.0 + 0 as f32) % WIDTH as f32,
        50.0 + (0 % 2) as f32,
        BALL_RADIUS,
        0,
    );

    let mut ctx = 0;

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_fps(0, 0);
        ctx = ctx + 1;

        game_state
        .add_ball(
            (250.0 + ctx as f32) % WIDTH as f32,
            55.0 + (ctx % 2) as f32,
            BALL_RADIUS,
            ctx,
        )
            .check_for_collisions()
            .linear_impulse_resolution()
            .apply_movement()
            .draw_balls(&mut d);

        //game_state.monitor_drift();
    }
}

// make a struct for the game state
struct GameState {
    gravity: f32,
    moves: Vec<Move>,
    arena: Arena<Ball>,
    collisions: Vec<(generational_arena::Index, generational_arena::Index)>,
    balls: Vec<Rc<Ball>>,
    color_generator: ColorGenerator,
}

struct ColorGenerator {
    index: u8,
}

impl ColorGenerator {
    fn new() -> ColorGenerator {
        ColorGenerator { index: 0 }
    }

    fn consumme(&mut self) -> Color {
        self.index = ((self.index as usize + 11) % 254) as u8;
        if (self.index as usize) > 255 {
            self.index = 0;
        }
        let c = match self.index % 6 {
            0 => Color::new(255, 0, self.index, 255),
            1 => Color::new(255, self.index, self.index, 255),
            2 => Color::new(self.index, 255, 0, 255),
            3 => Color::new(0, 255, self.index, 255),
            4 => Color::new(0, self.index, 255, 255),
            5 => Color::new(self.index, 0, 255, 255),
            _ => Color::new(0, 0, 0, 255),
        };

        return c;
    }
}

#[derive(Debug, Copy, Clone)]
struct Ball {
    x: f32,
    y: f32,
    radius: f32,
    color: Color,
    velocity: Velocity,
    mass: f32,
    coeff_restitution: f32,
}

#[derive(Debug, Copy, Clone)]
struct Velocity {
    x: f32,
    y: f32,
}

impl Velocity {
    fn relative_velocity(&self, other: &Velocity) -> Velocity {
        return Velocity {
            x: self.x - other.x,
            y: self.y - other.y,
        };
    }

    fn nomal(&self) -> f32 {
        let norm = -1.0;
        return norm;
    }
}

impl Ball {
    fn debug(&self) -> String {
        return format!(
            "Ball {{ x: {}, y: {}, radius: {}}}",
            self.x, self.y, self.radius
        );
    }
}

impl GameState {
    fn add_ball(&mut self, x: f32, y: f32, radius: f32, ctx: i32) -> &mut GameState {
        if ctx % 10 != 0 {
            return self;
        }
        let color = self.color_generator.consumme();
        self.arena.insert(Ball {
            x: x,
            y: y,
            radius: radius,
            color: color,
            velocity: Velocity { x: 5.0, y: 0.0 },
            mass: BALL_MASS,
            coeff_restitution: BALL_COEFF_RESTITUTION,
        });
        return self;
    }

    fn apply_movement(&mut self) -> &mut GameState {
        for (idx, ball) in self.arena.iter_mut() {
            let ball_y = ball.y as isize;
            let ball_x = ball.x as isize;
            let ball_radius = ball.radius as isize;

            // limit the speed to avoid the ball from going through the walls
            if ball.velocity.y > 10.0 {
                ball.velocity.y = 9.0;
            }
            if ball.velocity.y < -10.0 {
                ball.velocity.y = -9.0;
            }
            if ball.velocity.x > 10.0 {
                ball.velocity.x = 9.0;
            }
            if ball.velocity.x < -10.0 {
                ball.velocity.x = -9.0;
            }

            if (ball_y +  ball_radius) >= HEIGHT {
                ball.velocity.y = - (ball.velocity.y).abs() * BALL_COEFF_RESTITUTION;
            } else if (ball_y -  ball_radius) < 0 {
                ball.velocity.y = (ball.velocity.y).abs() * BALL_COEFF_RESTITUTION;
            } else {
                ball.velocity.y += self.gravity * ACCELERATION;
            }

            // apply gravity and friction to the ball

            ball.y += (ball.velocity.y * FRICTION);

            if ball_x  +  ball_radius > WIDTH {
                ball.velocity.x = -ball.velocity.x.abs() * BALL_COEFF_RESTITUTION;
            } else if ball_x  -  ball_radius < 0 {
                ball.velocity.x = ball.velocity.x.abs() * BALL_COEFF_RESTITUTION;
            }
            // check for overflow and underflow
            ball.x += ball.velocity.x * FRICTION;
        }
        return self;
    }

    fn draw_balls(&mut self, d: &mut RaylibDrawHandle) -> &mut GameState {
        for (idx, ball) in self.arena.iter() {
            d.draw_circle_v(Vector2::new(ball.x, ball.y), ball.radius, ball.color);
        }
        return self;
    }

    fn check_for_collisions(&mut self) -> &mut GameState {
        for (idx, ball) in self.arena.iter() {
            for (jdx, ball2) in self.arena.iter() {
                if idx == jdx || jdx > idx {
                    continue;
                }
                let distance = ((ball.x - ball2.x).powi(2) + (ball.y - ball2.y).powi(2)).sqrt();
                if distance <= (2.0 * BALL_RADIUS) {
                    self.collisions.push((idx, jdx));
                }
            }
        }
        return self;
    }

    fn linear_impulse_resolution(&mut self) -> &mut GameState {
        for (idx, jdx) in &self.collisions {
            let (ball_a, ball_b) = self.arena.get2_mut(*idx, *jdx);

            let mut ball_a = ball_a.unwrap();
            let mut ball_b = ball_b.unwrap();

            let e = ball_a.coeff_restitution.min(ball_b.coeff_restitution);
            let numerator =
                -(1.0 + e) * ball_a.velocity.relative_velocity(&ball_b.velocity).nomal();
            let denominator = 1.0 / ball_a.mass + 1.0 / ball_b.mass;
            let impulse = numerator / denominator;
            let impulse = impulse / 2.0 ;
            ball_a.velocity.x += impulse / ball_a.mass * -1.0;
            ball_b.velocity.x += impulse / ball_b.mass * 1.0;

            ball_a.velocity.y += impulse / ball_a.mass * -1.0;
            ball_b.velocity.y += impulse / ball_b.mass * 1.0;
        }
        self.collisions.clear();

        return self;
    }

    fn apply_collisions(&mut self) -> &mut GameState {
        for (idx, jdx) in &self.collisions {
            let (ball_a, ball_b) = self.arena.get2_mut(*idx, *jdx);

            let mut ball_a = ball_a.unwrap();
            let mut ball_b = ball_b.unwrap();

            let x = ball_a.x - ball_b.x;
            let y = ball_a.y - ball_b.y;

            let x = 2.0 * BALL_RADIUS - x.abs();
            let y = 2.0 * BALL_RADIUS - y.abs();

            // if ball_b is higher than ball_a
            if ball_a.y > ball_b.y {
                ball_b.y -= (y / 2.0);
            } else {
                ball_a.y -= (y / 2.0);
                ball_b.y += (y / 2.0);
            }

            // if ball_b is to the right of ball_a
            if ball_a.x > ball_b.x {
                ball_b.x -= (x / 2.0);
                ball_a.y += (x / 2.0);
            } else {
                ball_a.y -= (x / 2.0);
                ball_b.x += (x / 2.0);
            }

            if (ball_a.x + ball_a.radius) >= WIDTH as f32 {
                ball_a.x = (WIDTH as f32 - ball_a.radius) as f32;
            }
            if (ball_a.y + ball_a.radius) >= HEIGHT as f32 {
                ball_a.y = (HEIGHT as f32 - ball_a.radius) as f32;
            }
            if (ball_b.x + ball_b.radius) >= WIDTH as f32 {
                ball_b.x = (WIDTH as f32 - ball_b.radius) as f32;
            }
            if (ball_b.y + ball_b.radius) >= HEIGHT as f32 {
                ball_b.y = (HEIGHT as f32 - ball_b.radius) as f32;
            }
            if (ball_a.x - ball_a.radius) <= 0.0 {
                ball_a.x = ball_a.radius;
            }
            if (ball_a.y - ball_a.radius) <= 0.0 {
                ball_a.y = ball_a.radius;
            }
            if (ball_b.x - ball_b.radius) <= 0.0 {
                ball_b.x = ball_b.radius;
            }
            if (ball_b.y - ball_b.radius) <= 0.0 {
                ball_b.y = ball_b.radius;
            }

            // make them move away from each other
        }
        self.collisions.clear();
        return self;
    }
}

// Make a grid of balls

struct Grid {
    width: usize,
    height: usize,
    rows: usize,
    cols: usize,
    spacing: f32,
    // make the matrix take an option of a reference of ball
}

#[derive(Debug)]
struct Move {
    from: (usize, usize),
    to: (usize, usize),
}
