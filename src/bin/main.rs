use iced::{widget, Container, Element, Length, Sandbox, Settings};
use rand::Rng;
use rgb::{alt::BGRA, ComponentBytes};
// use image;

#[derive(Debug, Copy, Clone, PartialEq)]
enum CellState {
    Alive,
    Dead,
}

#[derive(Debug, Copy, Clone)]
struct Cell {
    state: CellState,
    size: usize,
    color: BGRA<u8>,
    x: usize,
    y: usize,
}

impl Cell {
    fn new(state: CellState, size: usize, x: usize, y: usize) -> Cell {
        let color = match state {
            CellState::Alive => BGRA {
                r: 255,
                g: 0,
                b: 128,
                a: 255,
            },
            CellState::Dead => BGRA {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        };

        Cell {
            state,
            size,
            color,
            x,
            y,
        }
    }

    fn set_state(&mut self, state: &CellState) {
        self.state = *state;
        self.color = match state {
            CellState::Alive => BGRA {
                r: 255,
                g: 0,
                b: 128,
                a: 255,
            },
            CellState::Dead => BGRA {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        };
    }
}

pub fn main() -> iced::Result {
    Conway::run(Settings::default())
}

struct Conway {
    cell_size: usize, // Length/width of cell given in number of pixels
    board_size: usize, // Length/width of board given in number of cells
    cells: Vec<Vec<Cell>>,
    bytes_per_pixel: usize,
    pixel_bytes: Vec<u8>,
}

impl Sandbox for Conway {
    type Message = ();

    fn new() -> Conway {
        let cell_size = 12;
        let board_size = 64;
        let bytes_per_pixel = 4; // One byte for each BGRA value
        let pixel_bytes =
            vec![0; board_size * cell_size * board_size * cell_size * bytes_per_pixel];

        let mut cells: Vec<Vec<Cell>> = (0..board_size)
            .map(|y| {
                (0..board_size)
                    .map(|x| Cell::new(CellState::Dead, cell_size, x, y))
                    .collect()
            })
            .collect();

        // Randomly place a number of living cells on the grid
        let living_cell_percent = 20;
        let mut rng = rand::thread_rng();
        let mut live_cells: Vec<(usize, usize)> = (0..((board_size * board_size * living_cell_percent)/100))
            .map(|_| (rng.gen_range(0..board_size), rng.gen_range(0..board_size)))
            .collect(); // Start with ~10% random living cells

        live_cells.sort();
        live_cells.dedup();

        println!(
            "{}/{} ≈ {}% living cells.",
            live_cells.len(),
            cells.len() * cells[0].len(),
            (live_cells.len() as f64) / (cells.len() as f64 * cells[0].len() as f64) * 100.0
        );

        for (x, y) in live_cells {
            cells[y][x].set_state(&CellState::Alive)
        }


        Conway {
            cell_size,
            board_size,
            cells,
            bytes_per_pixel,
            pixel_bytes,
        }
    }

    fn title(&self) -> String {
        String::from("The Game of Life")
    }

    fn update(&mut self, _message: Self::Message) {
        let transitions: Vec<Vec<(usize, usize, CellState)>> = self
            .cells
            .iter()
            .map(|row| {
                row.iter()
                    .filter_map(|cell| {
                        let mut neighbor_states = vec![]; // Every cell has 8 neighbors
                        for x_offset in -1..=1_isize {
                            for y_offset in -1..=1_isize {
                                let x = (cell.x as isize).rem_euclid(x_offset) as usize;
                                let y = (cell.y as isize).rem_euclid(y_offset) as usize;

                                if !(x == y && x == 0) {
                                    neighbor_states.push(self.cells[y][x].state);
                                }
                            }
                        }
                        let live_neighbor_count = neighbor_states
                            .iter()
                            .filter(|&state| *state == CellState::Alive)
                            .count();

                        let x = cell.x;
                        let y = cell.y;

                        match cell.state {
                            CellState::Dead if live_neighbor_count == 3 => {
                                Some((x, y, CellState::Alive))
                            }
                            CellState::Alive => match live_neighbor_count {
                                0..=1 => Some((x, y, CellState::Dead)),
                                2..=3 => None,
                                _ => Some((x, y, CellState::Dead)),
                            },
                            _ => None,
                        }
                    })
                    .collect()
            })
            .collect();

        let transitions: Vec<(usize, usize, CellState)> =
            transitions.into_iter().flatten().collect();

        println!("{:#?}", transitions);

        for (x, y, state) in transitions {
            self.cells[y][x].set_state(&state);
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        // Consider using Piet for drawing and compare performance https://docs.rs/piet/0.3.0/piet/struct.ImageBuf.html
        // Update the pixel bytes
        let edge_color = BGRA {
            r: 0,
            g: 0,
            b: 0,
            a: 170,
        };

        let bytes_per_cell_edge = self.cell_size * self.bytes_per_pixel;
        for row in self.cells.iter() {
            for cell in row {
                for y_offset in 0..self.cell_size {
                    let pixels = if y_offset == 0 || y_offset == self.cell_size - 1 {
                        vec![edge_color; self.cell_size]
                    } else {
                        let mut pixels = vec![cell.color; self.cell_size];
                        pixels[0] = edge_color;
                        pixels[self.cell_size - 1] = edge_color;
                        pixels
                    };

                    // let pixels = vec![edge_color; self.cell_size]; // No grid edges
                    let bytes = pixels.as_bytes();

                    let y = cell.y * self.cell_size + y_offset;
                    let start =
                        y * self.board_size * bytes_per_cell_edge + cell.x * bytes_per_cell_edge;
                    let end = start + bytes_per_cell_edge;
                    self.pixel_bytes.splice(start..end, bytes.iter().cloned());
                }
            }
        }

        let image_dimensions = (self.board_size * self.cell_size) as u32;
        // image::save_buffer(r"C:\Users\Andy\Desktop\test.jpg", &self.pixel_bytes.clone(), image_dimensions, image_dimensions, image::ColorType::Bgra8).unwrap();
        let img = widget::image::Handle::from_pixels(
            image_dimensions,
            image_dimensions,
            self.pixel_bytes.clone(),
        );
        let img = widget::image::Image::new(img);

        Container::new(img)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .center_x()
            .center_y()
            .into()
    }
}
