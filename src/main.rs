#![recursion_limit = "20000"]

use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    cmp::{self, max, min},
    ops::{Deref, DerefMut},
    rc::Rc,
};

use generational_arena::Arena;
use raylib::prelude::*;

const HEIGHT: isize = 480;
const WIDTH: isize = 640;
const GRAVITY: f32 = 4.0;
const BALL_RADIUS: f32 = 20.0;
const FRICTION: f32 = 0.90;
const ACCELERATION: f32 = 0.5;
const MAX_SPEED: f32 = 20.0;
const BALL_MASS: f32 = 1.0;
const BALL_COEFF_RESTITUTION: f32 = 0.80;
const TARGET_FPS : u32 = 60; 

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Hello, World")
        .build();

    // limit the framerate to 60
    rl.set_target_fps(TARGET_FPS);

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
                55.0 + (ctx % 4) as f32,
                BALL_RADIUS,
                ctx,
            )
            .handle_collisions()
            .apply_gravity()
            .update_balls()
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

impl Ball {
    fn get_speed(&self) -> f32 {
        let dx = self.x - self.last_position_x;
        let dy = self.y - self.last_position_y;
        let speed = (dx * dx + dy * dy).sqrt();

        return speed;
    }
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
    mass: f32,
    coeff_restitution: f32,
    last_position_x: f32,
    last_position_y: f32,
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
            mass: BALL_MASS,
            coeff_restitution: BALL_COEFF_RESTITUTION,
            last_position_x: x - 10.0,
            last_position_y: y - 10.0,
        });
        return self;
    }

    fn apply_gravity(&mut self) -> &mut GameState {
        for (_, ball) in self.arena.iter_mut() {
            ball.y += self.gravity;
        }
        return self;
    }

    fn draw_balls(&mut self, d: &mut RaylibDrawHandle) -> &mut GameState {
        for (idx, ball) in self.arena.iter() {
            d.draw_circle_v(Vector2::new(ball.x, ball.y), ball.radius, ball.color);
        }
        return self;
    }

    fn handle_collisions(&mut self) -> &mut GameState {
        // make 8 substeps
        for (_, ball) in self.arena.iter_mut() {
            ball.last_position_x = ball.x;
            ball.last_position_y = ball.y;
        }
        let nb_iter : u32 =  8;
        // calculate the time step
        let dt = 1.0 / (TARGET_FPS as f32 * nb_iter as f32);
        
        for i in 1..nb_iter {
            self.check_for_collisions()
                .solve_collisions(dt)
                .handle_oob();
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

    /*
    fn linear_impulse_resolution(&mut self) -> &mut GameState {
        for (idx, jdx) in &self.collisions {
            let (ball_a, ball_b) = self.arena.get2_mut(*idx, *jdx);

            let mut ball_a = ball_a.unwrap();s
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
    */

    fn solve_collisions(&mut self, dt : f32) -> &mut GameState {
        for (idx, jdx) in &self.collisions {
            let (ball_a, ball_b) = self.arena.get2_mut(*idx, *jdx);

            let response_coef = BALL_COEFF_RESTITUTION;

            let mut ball_a = ball_a.unwrap();
            let mut ball_b = ball_b.unwrap();
            let  min_dist : f32 = ball_a.radius + ball_b.radius;

            let o2_o1 = Vector2::new(ball_a.x - ball_b.x, ball_a.y - ball_b.y);
            let dist2 = o2_o1.x.powi(2) + o2_o1.y.powi(2);
            let dist = dist2.sqrt();

            let delta = 0.5 * response_coef *  (dist - min_dist);
            let col_vec = o2_o1 / dist * delta ;

            if (dist2 > 0.2) {            
            // update old position
            ball_a.x -= col_vec.x;
            ball_a.y -= col_vec.y;

            ball_b.x += col_vec.x;
            ball_b.y += col_vec.y;
            }
        }
        self.collisions.clear();
        return self;
    }

    fn handle_oob(&mut self) -> &mut GameState {
        for (_, ball) in self.arena.iter_mut() {
            if ball.x - BALL_RADIUS < 0.0 {
                ball.x = 0.0 + BALL_RADIUS;
            }
            if ball.x + BALL_RADIUS > WIDTH as f32 {
                ball.x = WIDTH as f32 - BALL_RADIUS;
            }
            if ball.y - BALL_RADIUS < 0.0 {
                ball.y = 0.0 + BALL_RADIUS;
            }
            if ball.y + BALL_RADIUS > HEIGHT as f32 {
                ball.y = HEIGHT as f32 - BALL_RADIUS;
            }
        }
        return self;
    }

    fn update_balls(&mut self) -> &mut GameState {
        for (_, ball) in self.arena.iter_mut() {
            

            // if ball.get_speed() < 1.0 {
            //     ball.x = ball.last_position_x;
            //     ball.y = ball.last_position_y;
            // }
        }
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
