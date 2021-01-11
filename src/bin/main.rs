// TODO:
// 1: Add Grid
// 2: Add Speed slider

mod simulation;
use simulation::{Cell, CellState, Position};
mod util; // Contains channels for inter-thread communication

use iced::{
    canvas::{self, Cache, Canvas, Cursor, Frame, Geometry},
    executor, time, Application, Command, Container, Element, Length, Point, Rectangle,
    Settings, Size, Subscription,
};

use std::thread;
use std::time::{Duration, Instant};

struct UI {
    backend: util::ThreadChannel<simulation::Message>,
    cell_grid: CellGrid,
    target_refresh_rate: u64,
}

struct Controls {}

// Types of messages that can be sent between UI functions
#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
}

impl Application for UI {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (UI, Command<Self::Message>) {
        let cell_size = 8;
        let grid_size = 96;
        let target_refresh_rate = 60; // Please don't set this to 0
        let evolution_rate = 3000; // evolutions/(100s)
        let (ui, backend) = util::ThreadChannel::new_pair();

        thread::Builder::new()
            .name("Game of Life Simulation".to_string())
            .spawn(move || {
                let mut simulation =
                    simulation::Simulation::new(ui, grid_size, target_refresh_rate, evolution_rate);
                simulation.run();
            })
            .unwrap(); // Not sure what to do here besides this unwrap, as I'm not the one calling this outer function

        let ui = UI {
            backend,
            cell_grid: CellGrid::new(cell_size, grid_size),
            target_refresh_rate,
        };

        (ui, Command::none())
    }

    fn title(&self) -> String {
        String::from("Conway's Game of Life")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Tick(_) => {
                let backend_updates = self.backend.receive();
                if !backend_updates.is_empty() {
                    self.cell_grid.frame_content.clear();
                }
                for update in backend_updates {
                    match update {
                        simulation::Message::CellTransitions(transitions) => {
                            for (position, state) in transitions {
                                self.cell_grid.cells[position.y][position.x].set_state(state);
                            }
                        }
                    }
                }
            }
        }

        // Async command thingy. No touchy.
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        let canvas = Canvas::new(&self.cell_grid)
            .width(Length::Fill)
            .height(Length::Fill);

        Container::new(canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_micros(1_000_000 / self.target_refresh_rate)).map(Message::Tick)
    }
}

pub fn main() -> iced::Result {
    UI::run(Settings::default())
}

struct CellGrid {
    cell_size: usize, // Edge length of cell in pixels
    grid_size: usize, // Edge length of grid in cells
    cells: Vec<Vec<Cell>>,
    frame_content: Cache,
}

impl CellGrid {
    fn new(cell_size: usize, grid_size: usize) -> Self {
        let cells: Vec<Vec<Cell>> = (0..grid_size)
            .map(|y| {
                (0..grid_size)
                    .map(|x| Cell::new(CellState::Dead, Position { x, y }))
                    .collect()
            })
            .collect();

        Self {
            cell_size,
            grid_size,
            cells,
            frame_content: Cache::new(),
        }
    }
}

impl canvas::Program<Message> for &CellGrid {
    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let frame_conent = self.frame_content.draw(bounds.size(), |frame| {
            for row in &self.cells {
                for cell in row {
                    cell.draw(frame, self.cell_size);
                }
            }
        });

        vec![frame_conent]
    }
}

impl Cell {
    fn draw(&self, frame: &mut Frame, size: usize) {
        let top_left = Point::from(self.position * size);
        let size = size as f32;
        let size = Size {
            width: size,
            height: size,
        };
        frame.fill_rectangle(top_left, size, self.color);
    }
}
