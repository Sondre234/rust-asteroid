use bevy::math::ops::abs;
use bevy::math::ops::{cos, sin};
use macroquad::prelude::*;
use std::cmp::Ordering;

const PLAYER_COLOR: Color = GRAY;
const PLAYER_SPEED: f32 = 0.03;

const ASTEROID_COLOR: Color = WHITE;
const ASTEROID_RADIUS: f32 = 30.0;
const ASTEROID_THICKNESS: f32 = 2.0;
const ASTEROID_SPEED: f32 = 1.0;

const BULLET_SIZE: f32 = 1.0;
const BULLET_COLOR: Color = GREEN;
const BULLET_SPEED: f32 = 10.0; // Lower = faster

static HEALTH: std::sync::Mutex<i32> = std::sync::Mutex::new(10);

#[macroquad::main("")]
async fn main() {
    clear_background(BLACK);

    let mut bullets: Vec<Bullet> = Vec::new();
    let mut asteroids: Vec<Asteroid> = Vec::new();

    let mut player = Player::default_player();
    let mut game = Game::new();

    loop {
        game.update(&mut asteroids);

        check_collisions(&mut bullets, &mut asteroids, &player);
        Bullet::bullet_logic(&mut bullets, &player);
        Asteroid::asteroid_logic(&mut asteroids, &player);
        if is_key_down(KeyCode::Right) {
            player = player.rotate_cw();
        }
        if is_key_down(KeyCode::Left) {
            player = player.rotate_ccw();
        }

        Asteroid::check(&mut asteroids, &player);
        player.draw_player();
        next_frame().await;
    }
}
struct Player {
    x: Vec2,
    y: Vec2,
    z: Vec2,
}
struct Point {
    x: f32,
    y: f32,
}

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
struct Direction {
    x: f32,
    y: f32,
}

struct Game {
    timer: f32,
    interval: f32,
}

impl Game {
    fn new() -> Self {
        Self {
            timer: 0.0,
            interval: 0.5,
        }
    }

    fn update(&mut self, asteroids: &mut Vec<Asteroid>) {
        self.timer += get_frame_time();
        if self.timer >= self.interval {
            self.timer -= self.interval;
            self.increase_time(asteroids);
        }
    }

    fn increase_time(&mut self, asteroids: &mut Vec<Asteroid>) {
        dbg!("+.5sec");
        let spawn_x = rand::gen_range(0, screen_width() as i32) as f32;
        let spawn_y = rand::gen_range(0, screen_height() as i32) as f32;

        let asteroid = Asteroid::new_asteroid(spawn_x, spawn_y);
        asteroids.push(asteroid)
    }
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
        }
    }

    fn parse_x(&self) -> Point {
        // can also iter and give all but only need x so... too bad!
        let x = self.x.x;
        let y = self.x.y;
        Point { x, y }
    }
    fn center(&self) -> Point {
        let cx = (self.x.x + self.y.x + self.z.x) / 3.0;
        let cy = (self.x.y + self.y.y + self.z.y) / 3.0;

        Point { x: cx, y: cy }
    }
    #[allow(unused)]
    fn iter(&self) -> impl Iterator<Item=&Vec2> {
        [&self.x, &self.y, &self.z].into_iter()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item=&mut Vec2> {
        [&mut self.x, &mut self.y, &mut self.z].into_iter()
    }

    fn rotate(mut self, theta: f32) -> Self {
        let cx = (self.x.x + self.y.x + self.z.x) / 3.0;
        let cy = (self.x.y + self.y.y + self.z.y) / 3.0;
        let cos = cos(theta);
        let sin = sin(theta);

        for curr in self.iter_mut() {
            let (dx, dy) = (curr.x - cx, curr.y - cy);
            let nx = cx + dx * cos - dy * sin;
            let ny = cy + dx * sin + dy * cos;
            curr.x = nx;
            curr.y = ny;
        }

        self
    }

    fn rotate_cw(self) -> Self {
        self.rotate(PLAYER_SPEED)
    }
    fn rotate_ccw(self) -> Self {
        self.rotate(-PLAYER_SPEED)
    }

    fn draw_player(&self) {
        draw_triangle(self.x, self.y, self.z, PLAYER_COLOR);
    }
}


impl Asteroid {
    fn check(asteroids: &mut Vec<Asteroid>, player: &Player) {
        for asteroid in asteroids.iter_mut() {
            let center = Vec2 { x: asteroid.x, y: asteroid.y };
            if is_triangle_touching_circle(center, ASTEROID_RADIUS, player.x, player.y, player.x) {
                lose_life()
            }
        }
    }
    fn asteroid_logic(asteroids: &mut Vec<Asteroid>, player: &Player) {
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
    fn new_asteroid(x: f32, y: f32) -> Self {
        Asteroid {
            x,
            y,
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
                bullet.x += bullet.direction.x / BULLET_SPEED;
                bullet.y += bullet.direction.y / BULLET_SPEED;
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

fn check_collisions(bullets: &mut Vec<Bullet>, asteroids: &mut Vec<Asteroid>, player: &Player) {
    // doing it this way nukes performance, small program so who cares... too bad!
    for asteroid in asteroids.iter_mut() {
        let center = Vec2 { x: asteroid.x, y: asteroid.y };
        if is_triangle_touching_circle(center, ASTEROID_RADIUS, player.x, player.y, player.x) {
            dbg!("True");
            asteroid.live = false;
        }
        for bullet in bullets.iter_mut() {
            if hit_asteroid(&bullet, &asteroid) {
                bullet.live = false;
                asteroid.live = false;
            }
        }
    }
}
fn hit_asteroid(bullet: &Bullet, asteroid: &Asteroid) -> bool {
    if !bullet.live || !asteroid.live {
        return false;
    }
    let distance_x = abs(bullet.x - asteroid.x);
    let distance_y = abs(bullet.y - asteroid.y);

    distance_x < ASTEROID_RADIUS && distance_y < ASTEROID_RADIUS
}
fn dist_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let l2 = a.distance_squared(b);
    if l2 == 0.0 { return p.distance(a); }

    let t = ((p.x - a.x) * (b.x - a.x) + (p.y - a.y) * (b.y - a.y)) / l2;
    let t = t.clamp(0.0, 1.0);
    let projection = a + t * (b - a);

    p.distance(projection)
}
fn is_point_in_triangle(p: Vec2, v1: Vec2, v2: Vec2, v3: Vec2) -> bool {
    let sign = |p1: Vec2, p2: Vec2, p3: Vec2| {
        (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
    };

    let d1 = sign(p, v1, v2);
    let d2 = sign(p, v2, v3);
    let d3 = sign(p, v3, v1);

    let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
    let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);

    !(has_neg && has_pos)
}

fn is_triangle_touching_circle(center: Vec2, radius: f32, v1: Vec2, v2: Vec2, v3: Vec2) -> bool {
    if dist_to_segment(center, v1, v2) <= radius { return true; }
    if dist_to_segment(center, v2, v3) <= radius { return true; }
    if dist_to_segment(center, v3, v1) <= radius { return true; }

    if is_point_in_triangle(center, v1, v2, v3) { return true; }

    false
}


fn lose_life() {
    let mut lives = HEALTH.lock().expect("Error acquiring lock");

    if *lives > 0 {
        *lives -= 1;
        println!("Lost life, Remaining lives: {}", *lives);
    } else {
        println!("Game over!");
        std::process::exit(1);
    }
}