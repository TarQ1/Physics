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
use raylib::{prelude::*, ffi::sqrt};

const HEIGHT: isize = 480;
const WIDTH: isize = 640;
const GRAVITY: f32 = 1.0;
const BALL_RADIUS: f32 = 10.0;
const FRICTION: f32 = 0.90;
const ACCELERATION: f32 = 0.5;
const MAX_SPEED: f32 = 20.0;
const BALL_MASS: f32 = 1.0;
const BALL_COEFF_RESTITUTION: f32 = 0.8;
const TARGET_FPS : f32 = 15.0;
const PADDING : f32 = BALL_RADIUS / 10.0;
fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Hello, World")
        .build();

    // limit the framerate to 60
    rl.set_target_fps(TARGET_FPS as u32);

    let arena: Arena<Ball> = Arena::new();

    let color_generator = ColorGenerator::new();

    let mut game_state = GameState {
        gravity: GRAVITY,
        moves: vec![],
        arena: arena,
        collisions: vec![],
        balls: vec![],
        color_generator: color_generator,
    };

    let mut ctx = 0;
    let dt = 1.0 / TARGET_FPS;



    

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_fps(0, 0);
        ctx = ctx + 1;

        game_state
            // .add_ball(20.0, 10.0, BALL_RADIUS, ctx)
            .apply_movement(dt)
            .check_for_collisions()
            .apply_collisions()
            .draw_balls(&mut d);

        if ctx == 20
        {
            game_state.arena.insert(Ball {
                position: Vector2{ x: 105.0, y: 510.0 },
                old_position:  Vector2{ x: 100.0, y: 515.0 },
                radius: BALL_RADIUS,
                color: Color::RED,
                mass: BALL_MASS,
                coeff_restitution: BALL_COEFF_RESTITUTION,
            });
        
            game_state.arena.insert(Ball {
                position: Vector2{ x: 310.0, y: 510.0 },
                old_position:  Vector2{ x: 315.0, y: 515.0 },
                radius: BALL_RADIUS,
                color: Color::BLUE,
                mass: BALL_MASS,
                coeff_restitution: BALL_COEFF_RESTITUTION,
            });
        }
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
    position: Vector2,
    old_position: Vector2,
    radius: f32,
    color: Color,
    mass: f32,
    coeff_restitution: f32,
}

impl Ball {
    fn debug(&self) -> String {
        return format!(
            "Ball {{ x: {}, y: {}, radius: {}}}",
            self.position.x, self.position.y, self.radius
        );
    }

    fn update_position(&mut self, pos : Vector2) -> &mut Ball {
        self.old_position = self.position;
        self.position = pos;
        return self;
    }
}

impl GameState {
    fn add_ball(&mut self, x: f32, y: f32, radius: f32, ctx: i32) -> &mut GameState {
        if ctx % 20 != 0 {
            return self;
        }
        let color = self.color_generator.consumme();
        self.arena.insert(Ball {
            position: Vector2{ x: 20.0, y: 20.0 },
            old_position:  Vector2{ x: 10.0, y: 10.0 },
            radius: radius,
            color: color,
            mass: BALL_MASS,
            coeff_restitution: BALL_COEFF_RESTITUTION,
        });
        return self;
    }

    fn apply_movement(&mut self, dt : f32) -> &mut GameState {
        for (idx, ball) in self.arena.iter_mut() {
            let ball_radius = ball.radius;
            let mut acceleration = (ball.position - ball.old_position) * 0.5;

            if ball.position.y + ball_radius > HEIGHT as f32 {
                acceleration.y = -acceleration.y;
                acceleration *= BALL_COEFF_RESTITUTION;
                ball.position.y = HEIGHT as f32 - ball_radius;
            } else if (ball.position.y - ball_radius) < 0.0 {
                acceleration.y = -acceleration.y;
                acceleration *= BALL_COEFF_RESTITUTION;
                ball.position.y = 0.0 + ball_radius
            } else {
            }
            if ball.position.x + ball_radius > WIDTH as f32 {
                acceleration.x = -acceleration.x;
                acceleration *= BALL_COEFF_RESTITUTION;
                ball.position.x = WIDTH as f32 - ball_radius
            } else if ball.position.x - ball_radius < 0.0 {
                acceleration.x = -acceleration.x;
                acceleration *= BALL_COEFF_RESTITUTION;
                ball.position.x = 0.0 + ball_radius;
            }

            let acceleration = acceleration + acceleration; // + (Vector2{x: 0.0, y: GRAVITY});
            
            ball.update_position(Vector2::new(ball.position.x + acceleration.x, ball.position.y + acceleration.y));

        }
        return self;
    }

    fn draw_balls(&mut self, d: &mut RaylibDrawHandle) -> &mut GameState {
        for (idx, ball) in self.arena.iter() {
            d.draw_circle_v(ball.position, ball.radius, ball.color);
        }
        return self;
    }

    fn check_for_collisions(&mut self) -> &mut GameState {
        for (idx, ball) in self.arena.iter() {
            for (jdx, ball2) in self.arena.iter() {
                if idx == jdx || jdx > idx {
                    continue;
                }
                let distance = ((ball.position.x - ball2.position.x).powi(2) + (ball.position.y - ball2.position.y).powi(2)).sqrt();
                if distance <= (2.0 * BALL_RADIUS){
                    self.collisions.push((idx, jdx));
                }
            }
        }
        return self;
    }

    fn apply_collisions(&mut self) -> &mut GameState {
        for (idx, jdx) in &self.collisions {
            let (ball_a, ball_b) = self.arena.get2_mut(*idx, *jdx);

            let ball_a = ball_a.unwrap();
            let ball_b = ball_b.unwrap();

            let dx = ball_a.position.x - ball_b.position.x;
            let dy = ball_a.position.y - ball_b.position.y;

            let ball_a_accel = ball_a.position - ball_a.old_position;
            let ball_b_accel = ball_b.position - ball_b.old_position;

            let mut cd = dx*dx + dy*dy;
            let r = ball_a.radius + ball_b.radius;

            if cd <= r*r {
                cd = cd.sqrt();
                let cr = r - cd;

                let dx = dx / cd * 0.5;
                let dy =  dy / cd * 0.5;

                ball_a.update_position(Vector2::new(ball_a.position.x + dx * cr, ball_a.position.y + dy * cr));
                ball_b.update_position(Vector2::new(ball_b.position.x - dx * cr, ball_b.position.y - dy * cr));


            }
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
