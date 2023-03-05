use std::{
    borrow::BorrowMut,
    cell::RefCell,
    cmp::{self, max, min},
    rc::Rc,
};

use raylib::prelude::*;

const HEIGHT: usize = 480;
const WIDTH: usize = 640;
const GRAVITY: f32 = 2.0;
const BALL_RADIUS: f32 = 10.0;
const FRICTION: f32 = 0.95;
const ACCELERATION: f32 = 0.1;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Hello, World")
        .build();

    // limit the framerate to 60
    rl.set_target_fps(10);

    let mut matrix: Vec<Vec<Option<Ball>>> = vec![];

    // initialize rows

    for _ in 0..((HEIGHT as f32 / BALL_RADIUS) as usize) {
        let mut row: Vec<Option<Ball>> = vec![];
        for _ in 0..((WIDTH as f32 / BALL_RADIUS) as usize) {
            row.push(None);
        }
        matrix.push(row);
    }

    // create a game state
    let grid: Grid = Grid {
        width: WIDTH,
        height: HEIGHT,
        rows: (HEIGHT as f32 / BALL_RADIUS) as usize,
        cols: (WIDTH as f32 / BALL_RADIUS) as usize,
        spacing: 10.0,
        // create an empty matrix with rows= (HEIGHT as f32 / BALL_RADIUS) & cols= (WIDTH as f32 / BALL_RADIUS)
        // fill the matric of None
        matrix: matrix,
    };

    let mut game_state = GameState {
        gravity: GRAVITY,
        grid: grid,
        moves: vec![],
        collisions: vec![],
    };

    let mut ctx = 0;

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_fps(0, 0);
        ctx = ctx + 1;

        game_state
            .apply_gravity()
            .update_position()
            .check_for_collisions()
            .apply_collisions()
            .apply_moves()
            .add_ball(
                (250.0 + ctx as f32) % WIDTH as f32,
                50.0 + (ctx % 2) as f32,
                BALL_RADIUS,
                Color::RED,
                ctx,
            )
            .draw_balls(&mut d);

        //game_state.monitor_drift();
    }
}

// make a struct for the game state
struct GameState {
    gravity: f32,
    grid: Grid,
    moves: Vec<Move>,
    collisions: Vec<(Ball, Ball)>,
}

#[derive(Debug, Copy, Clone)]
struct Ball {
    x: f32,
    y: f32,
    radius: f32,
    color: Color,
    movement: Movement,
}

#[derive(Debug, Copy, Clone)]
struct Movement {
    x: f32,
    y: f32,
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
    fn add_ball(&mut self, x: f32, y: f32, radius: f32, color: Color, ctx: i32) -> &mut GameState {
        if ctx % 10 != 0 {
            return self;
        }
        self.grid.matrix[0][0] = Some(Ball {
            x: x,
            y: y,
            radius: radius,
            color: color,
            movement: Movement { x: 0.0, y: 0.0 },
        });

        return self;
    }

    fn apply_gravity(&mut self) -> &mut GameState {
        for vec in &mut self.grid.matrix {
            for ball_option in vec {
                match ball_option {
                    Some(ball) => {
                        // check for out of bounds
                        if ball.y + ball.radius >= self.grid.height as f32 {
                            continue;
                        }
                        // ball.movement.y += self.gravity;
                        // ball.y += ball.movement.y * ACCELERATION;
                        // if (ball.y + ball.radius) >= (self.grid.height - 1) as f32 {
                        //     ball.y = self.grid.height as f32 - ball.radius;
                        //     ball.movement.y = 0.0;
                        // }
                    }
                    None => (),
                }
            }
        }
        return self;
    }

    fn update_position(&mut self) -> &mut GameState {
        // update the position of the ball and put them in the corresponding cell
        self.moves.clear();
        for (idx, vec) in self.grid.matrix.iter().enumerate() {
            // get the indices of the ball
            for (jdx, ball_option) in vec.iter().enumerate() {
                match ball_option {
                    Some(ball) => {
                        let row = (ball.y / BALL_RADIUS) as isize;
                        let col = (ball.x / BALL_RADIUS) as isize;

                        // check if the ball is out of the grid
                        if row >= self.grid.rows as isize
                            || col >= self.grid.cols as isize
                            || row < 0
                            || col < 0
                        {
                            println!("Ball out of bounds");
                            continue;
                        }

                        // check if the cell is empty
                        if self.grid.matrix[row as usize][col as usize].is_none() {
                            self.moves.push(Move {
                                from: (idx, jdx),
                                to: (row as usize, col as usize),
                            });
                        }
                    }
                    None => (),
                }
            }
        }
        return self;
    }

    fn draw_balls(&mut self, d: &mut RaylibDrawHandle) -> &mut GameState {
        for vec in &self.grid.matrix {
            for ball_option in vec {
                match ball_option {
                    Some(ball) => {
                        d.draw_circle(ball.x as i32, ball.y as i32, ball.radius, ball.color)
                    }
                    None => (),
                }
            }
        }
        return self;
    }

    fn apply_moves(&mut self) -> &mut GameState {
        for mov in &self.moves {
            let (from_row, from_col) = mov.from;
            let (to_row, to_col) = mov.to;

            let rest = std::mem::replace(&mut self.grid.matrix[from_row][from_col], None);
            self.grid.matrix[to_row][to_col] = rest;
        }
        self.moves.clear();
        return self;
    }

    fn monitor_drift(&self) {
        // print the values of the balls and their grid position to check for drift
        for (idx, vec) in self.grid.matrix.iter().enumerate() {
            for (jdx, ball_option) in vec.iter().enumerate() {
                match ball_option {
                    Some(ball) => {
                        println!("drifto: {} {} {}", ball.debug(), idx, jdx);
                    }
                    None => (),
                }
            }
        }
    }

    fn check_for_collisions(&mut self) -> &mut GameState {
        self.collisions.clear();
        for (idx, vec) in (&self.grid.matrix).iter().enumerate() {
            // get the indices of the ball
            for (jdx, ball_option) in vec.iter().enumerate() {
                match ball_option {
                    Some(ball) => {
                        let row = (ball.y / BALL_RADIUS) as isize;
                        let col = (ball.x / BALL_RADIUS) as isize;

                        // check the adjacent cells
                        for i in -1..2 {
                            for j in -1..2 {
                                let new_row = row + i;
                                let new_col = col + j;

                                if new_row < 0 || new_col < 0 {
                                    continue;
                                }

                                let new_row = new_row as usize;
                                let new_col = new_col as usize;

                                if new_row >= self.grid.rows - 1 || new_col >= self.grid.cols - 1 {
                                    continue;
                                }
                                match &self.grid.matrix[new_row][new_col] {
                                    Some(other_ball) => {
                                        if ball.y > other_ball.y || ball.x > other_ball.x {
                                            continue;
                                        }
                                        if (ball.x, ball.y) != (other_ball.x, other_ball.y) {
                                            let distance = ((ball.x - other_ball.x).powi(2)
                                                + (ball.y - other_ball.y).powi(2))
                                            .sqrt();
                                            if distance <= (2.0 * BALL_RADIUS) {
                                                println!(
                                                    "added collision for {} and {}",
                                                    ball.debug(),
                                                    other_ball.debug()
                                                );
                                                self.collisions.push((*ball, *other_ball));
                                            }
                                        }
                                    }

                                    _ => (),
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
        println!("collisions: {:?}", self.collisions);
        println!("===================");
        return self;
    }

    fn apply_collisions(&mut self) -> &mut GameState {
        for (ball, other_ball) in &self.collisions {
            // move the ball away from each center
            let distance = ((ball.x - other_ball.x).powi(2) + (ball.y - other_ball.y).powi(2)).sqrt();

            let x_diff = (ball.x - other_ball.x);
            let y_diff = (ball.y - other_ball.y);

            let mut mut_ball = match self.grid.matrix[(ball.y / BALL_RADIUS) as usize]
                [(ball.x / BALL_RADIUS) as usize]
                .as_mut()
            {
                Some(ball) => ball,
                None => continue,
            };

            println!("{}", x_diff);

            if ball.x >= other_ball.x {
                mut_ball.x = (ball.x + x_diff);
            } else {
                mut_ball.x = (ball.x - x_diff);
            }

            if ball.y >= other_ball.y {
                mut_ball.y = (ball.y - y_diff);
            } else {
                mut_ball.y = (ball.y + y_diff);
            }

            let mut other_ball = match self.grid.matrix[(other_ball.y / BALL_RADIUS) as usize]
                [(other_ball.x / BALL_RADIUS) as usize]
                .as_mut()
            {
                Some(other_ball) => other_ball,
                None => continue,
            };

            if ball.x >= other_ball.x {
                other_ball.x = (other_ball.x - x_diff);
            } else {
                other_ball.x = (other_ball.x + x_diff);
            }

            if ball.y >= other_ball.y {
                other_ball.y = (other_ball.y + y_diff);
            } else {
                other_ball.y = (other_ball.y - y_diff);
            }
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
    matrix: Vec<Vec<Option<Ball>>>,
}

#[derive(Debug)]
struct Move {
    from: (usize, usize),
    to: (usize, usize),
}
