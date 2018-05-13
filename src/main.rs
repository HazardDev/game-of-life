extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;

pub struct Game {
    gl: GlGraphics,
    board: Board,
    running: bool,
}

impl Game {
    fn render(&mut self, arg: &RenderArgs) {
        use graphics;

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

        if let Some(m) = mouse_pos {
            if btn == &Button::Mouse(MouseButton::Left) {
                let x_across = (m[0] / self.board.scale) as usize;
                let y_down = (m[1] / self.board.scale) as usize;

                if x_across < self.board.tiles.len() && y_down < self.board.tiles[x_across].len() {
                    self.board.tiles[x_across][y_down] = match self.board.tiles[x_across][y_down] {
                        Tile::Alive => Tile::Dead,
                        Tile::Dead => Tile::Alive,
                    };
                }
            }
        }
    }

    fn resize_board(&mut self, window_dimensions: [u32; 2]) {
        self.board.resize_board(window_dimensions);
    }
}

struct Board {
    tiles: Vec<Vec<Tile>>,
    scale: f64,
    tile_width: i32,
    tile_height: i32,
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
            scale: scale as f64,
            tile_width: (window_width / scale) as i32,
            tile_height: (window_height / scale) as i32,
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
            scale: scale as f64,
            tile_width: (window_width / scale) as i32,
            tile_height: (window_height / scale) as i32,
            tiles: starting_tiles,
        }
    }

    fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs) {
        // println!("Board Render");

        let white: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        let black: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        let mut squares: Vec<Vec<graphics::types::Rectangle>> = Vec::new();

        self.tiles.iter().enumerate().for_each(|(x, tile_list)| {
            let mut this_list = Vec::new();

            tile_list.iter().enumerate().for_each(|(y, _)| {
                this_list.push(graphics::rectangle::square(
                    x as f64 * self.scale as f64,
                    y as f64 * self.scale as f64,
                    self.scale,
                ));
            });

            squares.push(this_list);
        });

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
        self.tiles = self.tiles
            .iter()
            .enumerate()
            .map(|(x_across, tile_array)| {
                tile_array
                    .iter()
                    .enumerate()
                    .map(|(y_down, &tile)| {
                        let adjacent_tiles =
                            self.get_adjacent_tiles(x_across as i32, y_down as i32);

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

    fn resize_board(&mut self, window_dimensions: [u32; 2]) {
        let scale = self.scale as u32;
        let window_width = window_dimensions[0];
        let window_height = window_dimensions[1];

        let mut new_tiles = Vec::new();

        let horizontal_capacity = (window_width / scale) as usize;
        let vertical_capacity = (window_height / scale) as usize;

        for width in 0..horizontal_capacity {
            new_tiles.push(Vec::new()); //Vec::with_capacity(horizontal_capacity + 1));

            for height in 0..vertical_capacity {
                if self.tiles.len() > width {
                    // println!("Tile Column Within Width: {}", width);
                    if self.tiles[width].len() > height {
                        // println!("Tile Within Height: {:?}", self.tiles[width][height]);
                        new_tiles[width as usize].push(self.tiles[width][height]);
                    } else {
                        new_tiles[width as usize].push(Tile::Dead);
                    }
                } else {
                    new_tiles[width as usize].push(Tile::Dead);
                }
            }
        }

        self.tiles = new_tiles;
        self.tile_height = vertical_capacity as i32;
        self.tile_width  = horizontal_capacity as i32;
    }

    fn get_adjacent_tiles(&self, x_across: i32, y_down: i32) -> i32 {
        let mut adjacent: i32 = 0;

        //Convert these checks to .get functions for S A F E T Y
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
    let opengl = OpenGL::V4_5;

    let window_width = 1000;
    let window_height = 750;
    let scale = 10;

    let mut window: Window = WindowSettings::new("game-of-life", [window_width, window_height])
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|err| {
            panic!(
                "Failed to build piston window. You may need to disable HDR. Full error: {}",
                err
            )
        });

    let board: Board = Board::new(scale, window_width, window_height);

    let mut game = Game {
        gl: GlGraphics::new(opengl),
        board: board,
        running: true,
    };

    let mut event_settings = EventSettings::new();
    event_settings.ups = 4;
    event_settings.max_fps = 250;

    let mut last_mouse_location: Option<[f64; 2]> = None;

    let mut events = Events::new(event_settings);
    while let Some(e) = events.next(&mut window) {
        if let Some(w) = e.resize_args() {
            game.resize_board(w);
        }

        if let Some(u) = e.update_args() {
            if !game.update(&u) {
                break;
            }
        }

        if let Some(r) = e.render_args() {
            game.render(&r);
        }

        if let Some(pos) = e.mouse_cursor_args() {
            last_mouse_location = Some(pos);
        }

        if let Some(s) = e.mouse_scroll_args() {
            let mut event_settings = events.get_event_settings();
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
