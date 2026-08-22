use bevy::math::ops::abs;
use bevy::{
    log::tracing_subscriber::field::debug,
    math::ops::{cos, sin},
};
use macroquad::input::KeyCode::Space;
use macroquad::prelude::*;
use std::cmp::Ordering;

const PLAYER_COLOR: Color = GRAY;
const PLAYER_SPEED: f32 = 0.01;

const ASTEROID_COLOR: Color = WHITE;
const ASTEROID_RADIUS: f32 = 10.0;
const ASTEROID_THICKNESS: f32 = 2.0;
const ASTEROID_SPEED: f32 = 0.5;

const BULLET_SIZE: f32 = 2.0;
const BULLET_COLOR: Color = GREEN;

const FRAME_RATE: f32 = 0.001;

#[macroquad::main("Main")]
async fn main() {
    clear_background(BLACK);

    let mut bullets: Vec<Bullet> = Vec::new();
    let mut asteroids: Vec<Asteroid> = Vec::new();

    let mut player = Player::default_player();
    let mut bullet: Bullet;
    let mut asteroid: Asteroid;

    let mut frames_ticked = 0.0;
    // game loop
    loop {
        asteroid = Asteroid::new_asteroid();

        Bullet::bullet_logic(&mut bullets, &player);
        Asteroid::asteroid_logic(&mut asteroids, &player);
        if is_key_down(KeyCode::Right) {
            player = player.rotate_cw();
        }
        if is_key_down(KeyCode::Left) {
            player = player.rotate_ccw();
        }

        frames_ticked += FRAME_RATE;
        player.draw_player();
        next_frame().await;
    }
}
struct Player {
    x: Vec2,
    y: Vec2,
    z: Vec2,
    angle: f32,
}
#[derive(Debug)]
struct Point {
    x: f32,
    y: f32,
}

#[derive(Clone)]
struct Asteroid {
    x: f32,
    y: f32,
    live: bool,
}
struct Bullet {
    x: f32,
    y: f32,
    direction: Direction,
    live: bool,
}
#[derive(Clone)]
struct Direction {
    x: f32,
    y: f32,
}
impl Player {
    fn default_player() -> Self {
        Player {
            x: Vec2 {
                x: screen_width() / 2.0 - 15.0,
                y: screen_height() / 2.0 - 30.0,
            },
            y: Vec2 {
                x: screen_width() / 2.0 - 15.0,
                y: screen_height() / 2.0 + 15.0,
            },
            z: Vec2 {
                x: screen_width() / 2.0 + 15.0,
                y: screen_height() / 2.0,
            },
            angle: PLAYER_SPEED,
        }
    }

    fn parse_x(&self) -> Point {
        // can also iter and give all but only need x so... too bad!
        let x = self.x.x;
        let y = self.x.y;
        Point { x, y }
    }
    fn parse_y(&self) -> Point {
        let x = self.y.x;
        let y = self.y.y;
        Point { x, y }
    }

    fn center(&self) -> Point {
        let cx = (self.x.x + self.y.x + self.z.x) / 3.0;
        let cy = (self.x.y + self.y.y + self.z.y) / 3.0;

        Point { x: cx, y: cy }
    }
    fn iter(&self) -> impl Iterator<Item=&Vec2> {
        [&self.x, &self.y, &self.z].into_iter()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item=&mut Vec2> {
        [&mut self.x, &mut self.y, &mut self.z].into_iter()
    }

    fn rotate(mut self, theta: f32) -> Self {
        let cx = (self.x.x + self.y.x + self.z.x) / 3.0;
        let cy = (self.x.y + self.y.y + self.z.y) / 3.0;
        let cos = cos(self.angle);
        let sin = sin(self.angle);

        for curr in self.iter_mut() {
            let (dx, dy) = (curr.x - cx, curr.y - cy);
            let nx = cx + dx * cos - dy * sin;
            let ny = cy + dx * sin + dy * cos;
            curr.x = nx;
            curr.y = ny;
        }

        self
    }
    fn rotate_cw(mut self) -> Self {
        self.rotate(1.0)
    }
    fn rotate_ccw(mut self) -> Self {
        self.rotate(-1.0)
    }

    fn draw_player(&self) {
        draw_triangle(self.x, self.y, self.z, PLAYER_COLOR);
    }
}


impl Asteroid {
    /*
    fn bullet_logic(bullets: &mut Vec<Bullet>, asteroid: &Vec<Asteroid>, player: &Player) {
        if is_key_pressed(KeyCode::Space) {
            let mut bullet: Bullet = Bullet::new_bullet(&player);
            bullet.direction = Bullet::direction(player.center(), player.x);
            bullets.push(bullet);
        }

        for bullet in &mut bullets.iter_mut() {
            if bullet.live {
                bullet.x += bullet.direction.x / 50.0;
                bullet.y += bullet.direction.y / 50.0;
                bullet.new();
            }
        }

        bullets.retain(|bullet| bullet.live == true)
    }
     */

    fn asteroid_logic(asteroids: &mut Vec<Asteroid>, player: &Player) {
        if is_key_pressed(KeyCode::LeftControl) {
            let mut asteroid = Asteroid::new_asteroid();
            asteroids.push(asteroid);
        }


        for asteroid in &mut asteroids.into_iter() {
            if !asteroid.live {
                continue;
            }
            asteroid.move_to_player(player.parse_x());
            asteroid.new();
        }

        asteroids.retain(|asteroid| {
            asteroid.live == true
        });
    }
    fn new_asteroid() -> Self {
        Asteroid {
            // x: screen_height(),
            // y: screen_width(),
            x: 600.0,
            y: 600.0,
            live: true,
        }
    }

    fn new(&mut self) {
        if self.live {
            draw_circle_lines(
                self.x,
                self.y,
                ASTEROID_RADIUS,
                ASTEROID_THICKNESS,
                ASTEROID_COLOR,
            );
        }
    }

    fn move_to_player(&mut self, target: Point) {
        if self.x > target.x {
            self.x -= ASTEROID_SPEED;
        } else {
            self.x += ASTEROID_SPEED;
        }
        if self.y > target.y {
            self.y -= ASTEROID_SPEED;
        } else {
            self.y += ASTEROID_SPEED;
        }
    }
}


impl Bullet {
    fn new_bullet(player: &Player) -> Self {
        Bullet {
            x: player.x.x,
            y: player.x.y,
            direction: Direction { x: 1.0, y: 1.0 },
            live: true,
        }
    }
    fn new(&mut self) {
        draw_circle(self.x, self.y, BULLET_SIZE, BULLET_COLOR);
    }

    fn bullet_logic(bullets: &mut Vec<Bullet>, player: &Player) {
        if is_key_pressed(KeyCode::Space) {
            let mut bullet: Bullet = Bullet::new_bullet(&player);
            bullet.direction = Bullet::direction(player.center(), player.x);
            bullets.push(bullet);
        }

        for bullet in &mut bullets.iter_mut() {
            if bullet.live {
                bullet.x += bullet.direction.x / 50.0;
                bullet.y += bullet.direction.y / 50.0;
                bullet.new();
            }
        }

        bullets.retain(|bullet| bullet.live == true)
    }

    fn direction(center: Point, player: Vec2) -> Direction {
        let mut dir: Direction = Direction { x: 0.0, y: 0.0 };
        let cx = center.x;
        let px = player.x;
        let cy = center.y;
        let py = player.y;

        match cx.partial_cmp(&px) {
            Some(Ordering::Greater) => dir.x = px - cx,
            Some(Ordering::Less) => dir.x = px - cx,
            Some(Ordering::Equal) => dir.x = px,
            None => {}
        }
        match cy.partial_cmp(&py) {
            Some(Ordering::Greater) => dir.y = py - cy,
            Some(Ordering::Less) => dir.y = py - cy,
            Some(Ordering::Equal) => dir.y = py,
            None => {}
        }

        dir
    }
}


fn hit_asteroid(bullet: &Bullet, asteroid: &Asteroid) -> bool {
    if !bullet.live || !asteroid.live {
        return false;
    }
    let distance_x = abs(bullet.x - asteroid.x);
    let distance_y = abs(bullet.y - asteroid.y);

    distance_x < 1.0 && distance_y < 1.0
}

fn hit_player(asteroid: &Asteroid, player: &Player) -> bool {
    if !asteroid.live {
        return false;
    }
    let distance_x = abs(player.parse_x().x - asteroid.x);
    let distance_y = abs(player.parse_y().y - asteroid.y);

    distance_x < 1.0 && distance_y < 1.0
}

fn calculate_hp() {

}
