extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;

const OPENGL: OpenGL = OpenGL::V4_5;

pub struct Game {
    gl: GlGraphics,
    board: Board,
    running: bool,
}

impl Game {
    fn render(&mut self, arg: &RenderArgs) {
        // let white: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        let black: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        self.gl.draw(arg.viewport(), |_c, gl| {
            graphics::clear(black, gl);
        });

        self.board.render(&mut self.gl, arg);
    }

    fn update(&mut self, arg: &UpdateArgs) -> bool {
        if !self.running {
            return true;
        }

        self.board.update(arg);

        true
    }

    fn input(&mut self, btn: &Button, mouse_pos: Option<[f64; 2]>) {
        if btn == &Button::Keyboard(Key::Return) || btn == &Button::Keyboard(Key::Space) {
            self.running = !self.running;
        }

        if btn == &Button::Keyboard(Key::R) {
            let new_board = Board::new_empty(self.board.scale, self.board.tile_width * self.board.scale, self.board.tile_height * self.board.scale);
            self.board = new_board;
        }

        if let Some(m) = mouse_pos {
            if btn == &Button::Mouse(MouseButton::Left) {
                let x_across = m[0] / self.board.scale as f64;
                let y_down = m[1] / self.board.scale as f64;

                self.board.tiles[x_across as usize][y_down as usize] =
                    match self.board.tiles[x_across as usize][y_down as usize] {
                        Tile::Alive => Tile::Dead,
                        Tile::Dead => Tile::Alive,
                    };
            }
        }
    }
}

struct Board {
    tiles: Vec<Vec<Tile>>,
    scale: u32,
    tile_width: u32,
    tile_height: u32,
}

impl Board {
    fn new(scale: u32, window_width: u32, window_height: u32) -> Board {
        let mut starting_tiles = Vec::new();

        for width in 0..window_width / scale {
            starting_tiles.push(Vec::with_capacity((window_height / scale) as usize));

            for _ in 0..window_height / scale {
                starting_tiles[width as usize].push(Tile::Dead);
            }
        }

        starting_tiles[(window_width / scale / 2) as usize][(window_height / scale / 2) as usize] =
            Tile::Alive;
        starting_tiles[(window_width / scale / 2 + 1) as usize]
            [(window_height / scale / 2) as usize] = Tile::Alive;
        starting_tiles[(window_width / scale / 2 + 2) as usize]
            [(window_height / scale / 2) as usize] = Tile::Alive;
        starting_tiles[(window_width / scale / 2 - 1) as usize]
            [(window_height / scale / 2 + 1) as usize] = Tile::Alive;
        starting_tiles[(window_width / scale / 2) as usize]
            [(window_height / scale / 2 + 1) as usize] = Tile::Alive;
        starting_tiles[(window_width / scale / 2 + 1) as usize]
            [(window_height / scale / 2 + 1) as usize] = Tile::Alive;

        Board {
            scale: scale,
            tile_width: (window_width / scale) as u32,
            tile_height: (window_height / scale) as u32,
            tiles: starting_tiles,
        }
    }

    #[allow(dead_code)]
    fn new_empty(scale: u32, window_width: u32, window_height: u32) -> Board {
        let mut starting_tiles = Vec::new();

        for width in 0..window_width / scale {
            starting_tiles.push(Vec::with_capacity((window_height / scale) as usize));

            for _ in 0..window_height / scale {
                starting_tiles[width as usize].push(Tile::Dead);
            }
        }

        Board {
            scale: scale,
            tile_width: window_width / scale,
            tile_height: window_height / scale,
            tiles: starting_tiles,
        }
    }

    fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs) {
        // println!("Board Render");

        let white: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        let black: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        let mut squares: Vec<Vec<graphics::types::Rectangle>> = Vec::new();

        for x in 0..self.tiles.len() {
            squares.push(Vec::new());

            for y in 0..self.tiles[x].len() {
                squares[x].push(graphics::rectangle::square(
                    x as f64 * self.scale as f64,
                    y as f64 * self.scale as f64,
                    self.scale as f64,
                ));
            }
        }

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;

            self.tiles.iter().enumerate().for_each(|(x, tile_array)| {
                tile_array.iter().enumerate().for_each(|(y, &tile)| {
                    let color = match tile {
                        Tile::Alive => white,
                        Tile::Dead => black,
                    };

                    graphics::rectangle(color, squares[x][y], transform, gl)
                });
            });
        });

        // println!("{:?}", self.tiles);
    }

    fn update(&mut self, _args: &UpdateArgs) -> bool {
        self.tiles = self
            .tiles
            .iter()
            .enumerate()
            .map(|(x_across, tile_array)| {
                tile_array
                    .iter()
                    .enumerate()
                    .map(|(y_down, &tile)| {
                        let adjacent_tiles =
                            self.get_adjacent_tiles(x_across as u32, y_down as u32);

                        let new_tile = match adjacent_tiles {
                            x if x < 2 => Tile::Dead,
                            x if (x == 2 || x == 3) && tile != Tile::Dead => Tile::Alive,
                            x if x > 3 => Tile::Dead,
                            x if x == 3 && tile == Tile::Dead => Tile::Alive,
                            _ => Tile::Dead,
                        };

                        new_tile
                    })
                    .collect()
            })
            .collect();

        true
    }

    fn get_adjacent_tiles(&self, x_across: u32, y_down: u32) -> u32 {
        let mut adjacent: u32 = 0;

        if x_across > 0 {
            //Top left
            if y_down > 0 && self.tiles[x_across as usize - 1][y_down as usize - 1] == Tile::Alive {
                adjacent += 1;
            }

            //Direct Left
            if self.tiles[x_across as usize - 1][y_down as usize] == Tile::Alive {
                adjacent += 1;
            }

            //Bottom Left
            if y_down < self.tile_height - 1
                && self.tiles[x_across as usize - 1][y_down as usize + 1] == Tile::Alive
            {
                adjacent += 1;
            }
        }

        if x_across < self.tile_width - 1 {
            //Top Right
            if y_down > 0 && self.tiles[x_across as usize + 1][y_down as usize - 1] == Tile::Alive {
                adjacent += 1;
            }

            //Direct Right
            if self.tiles[x_across as usize + 1][y_down as usize] == Tile::Alive {
                adjacent += 1;
            }
            //Bottom Right
            if y_down < self.tile_height - 1
                && self.tiles[x_across as usize + 1][y_down as usize + 1] == Tile::Alive
            {
                adjacent += 1;
            }
        }

        //Direct Top
        if y_down > 0 && self.tiles[x_across as usize][y_down as usize - 1] == Tile::Alive {
            adjacent += 1;
        }

        //Direct Bottom
        if y_down < self.tile_height - 1
            && self.tiles[x_across as usize][y_down as usize + 1] == Tile::Alive
        {
            adjacent += 1;
        }

        adjacent
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tile {
    Alive,
    Dead,
}

fn main() {
    let window_width = 1000;
    let window_height = 750;
    let scale = 10;

    let window = get_window(window_width, window_height);
    let board: Board = Board::new(scale, window_width, window_height);
    let game = Game {
        gl: GlGraphics::new(OPENGL),
        board: board,
        running: true,
    };

    let event_settings = get_event_settings();
    game_loop(game, event_settings, window)
}

fn get_window(width: u32, height: u32) -> Window {
    WindowSettings::new("game-of-life", [width, height])
        .opengl(OPENGL)
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|err| {
            panic!(
                "Failed to build piston window. You may need to disable HDR. Full error: {}",
                err
            )
        })
}

fn get_event_settings() -> EventSettings {
    let mut event_settings = EventSettings::new();
    event_settings.ups = 4;
    event_settings.max_fps = 250;

    event_settings
}

fn game_loop(mut game: Game, event_settings: EventSettings, mut window: Window) {
    let mut last_mouse_location: Option<[f64; 2]> = None;
    let mut events = Events::new(event_settings);
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            game.render(&r);
        }

        if let Some(u) = e.update_args() {
            if !game.update(&u) {
                break;
            }
        }

        match e {
            Event::Input(Input::Move(Motion::MouseCursor(x, y))) => {
                last_mouse_location = Some([x, y])
            }
            _ => {}
        }

        if let Some(s) = e.mouse_scroll_args() {
            let event_settings = events.get_event_settings();
            match s[1] as i32 {
                1 => events.set_ups(event_settings.ups + 1),
                -1 if event_settings.ups > 0 => events.set_ups(event_settings.ups - 1),
                _ => {}
            }
        }

        if let Some(b) = e.button_args() {
            if b.state == ButtonState::Press {
                game.input(&b.button, last_mouse_location);
            }
        }
    }
}
