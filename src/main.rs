#![warn(clippy::pedantic)]

use bracket_lib::prelude::*;

// Define Game Modes
#[derive(Debug)]
enum GameMode {
    Menu,    // Main Menu
    Playing, // Game is currently running
    End,     // Game is over
}

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 75.0;

// Dragon Frames
const DRAGON_FRAMES: [u16; 6] = [64, 1, 2, 3, 2, 1];

// Embed resources
embedded_resource!(TILE_FONT, "../resources/flappy32.png");

// Create a player struct
struct Player {
    x: i32,
    y: f32,
    velocity: f32,
    frame: usize, // Usize to index arrays
}

// Constructor for the player struct
impl Player {
    fn new(x: i32, y: i32) -> Player {
        Player {
            x: x,
            y: y as f32,
            velocity: 0.0,
            frame: 0,
        }
    }
    fn gravity_and_move(&mut self) {
        if self.velocity < 2.0 {
            self.velocity += 0.2;
        }
        self.y += self.velocity;
        self.x += 1;
        if self.y < 0.0 {
            self.y = 0.0;
        }
    }
    fn flap(&mut self) {
        self.velocity = -2.0;
    }
    fn render(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_fancy(
            PointF::new(0.0, self.y),
            1,
            Degrees::new(0.0),
            PointF::new(2.0, 2.0),
            WHITE,
            NAVY,
            DRAGON_FRAMES[self.frame],
        );
        ctx.set_active_console(0);
    }
}

// Create an obstacle struct
struct Obstacle {
    x: i32,
    gap_y: i32,
    size: i32,
}

// Constructor for the obstacle struct
impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle {
            x,
            gap_y: random.range(10, 40),
            size: i32::max(2, 20 - score),
        }
    }
    fn render(&mut self, ctx: &mut BTerm, player_x: i32) {
        let screen_x = self.x - player_x;
        let half_size = self.size / 2;

        // Draw the top half of the obstacle
        for y in 0..self.gap_y - half_size {
            ctx.set(screen_x, y, RED, BLACK, to_cp437('|'));
        }

        // Draw the bottom half of the obstacle
        for y in self.gap_y + half_size..self.gap_y + self.size {
            ctx.set(screen_x, y, RED, BLACK, to_cp437('|'));
        }
    }
    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        player.x == self.x
            && ((player.y as i32) < self.gap_y - half_size
                || player.y as i32 > self.gap_y + half_size)
    }
}

// Create State struct
struct State {
    player: Player,
    frame_time: f32,
    obstacle: Obstacle,
    mode: GameMode,
    score: i32,
}

// Constructor for State struct
impl State {
    fn new() -> Self {
        State {
            player: Player::new(5, 25), // Create a new player
            frame_time: 0.0,
            obstacle: Obstacle::new(SCREEN_WIDTH + 10, 0), // Create a new obstacle
            mode: GameMode::Menu,
            score: 0,
        }
    }
    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);
        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.player.gravity_and_move();
        }
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }
        self.player.render(ctx);
        ctx.print(0, 0, "Press space to flap");
        ctx.print(0, 1, &format!("Score: {}", self.score));
        self.obstacle.render(ctx, self.player.x);
        // Add points if the player ovoids the obstacle
        if self.player.x > self.obstacle.x {
            self.score += 1;
            self.obstacle = Obstacle::new(self.player.x + SCREEN_WIDTH, self.score);
        }
        if self.player.y as i32 > SCREEN_HEIGHT || self.obstacle.hit_obstacle(&self.player) {
            self.mode = GameMode::End;
        }
    }
    fn restart(&mut self) {
        self.player = Player::new(5, 25);
        self.frame_time = 0.0;
        self.obstacle = Obstacle::new(SCREEN_WIDTH, 0);
        self.mode = GameMode::Playing;
        self.score = 0;
    }
    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_color_centered(5, GREEN, BLACK, "Welcome to Flappy Dragon");
        ctx.print_color_centered(7, VIOLET, BLACK, "Press (P) to start");
        ctx.print_color_centered(9, RED, BLACK, "Press (Q) to quit");
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "You Died!");
        ctx.print_centered(6, &format!("Score: {}", self.score));
        ctx.print_centered(7, "Press (P) to restart");
        ctx.print_centered(9, "Press (Q) to quit");
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
}

// Trait for the state
impl GameState for State {
    // Init function
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            // TODO: Fill in this stub later
            GameMode::Menu => self.main_menu(ctx),
            GameMode::Playing => self.play(ctx),
            GameMode::End => self.dead(ctx),
        }
    }
}

fn main() -> BError {
    // Import the assets
    link_resource!(TILE_FONT, "resources/flappy32.png");

    let context = BTermBuilder::new()
        .with_font("flappy32.png", 32, 32)
        .with_simple_console(SCREEN_WIDTH, SCREEN_HEIGHT, "flappy32.png")
        .with_fancy_console(SCREEN_WIDTH, SCREEN_HEIGHT, "flappy32.png")
        .with_title("Flappy Dragon WebAssembly")
        .with_tile_dimensions(16, 16)
        .build()?;
    main_loop(context, State::new())
}
