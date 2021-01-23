// TODO:
// 1: Add Grid
// 2: Add Speed slider

mod simulation;
use simulation::{Cell, CellState, Position};
mod style;
mod util; // Contains channels for inter-thread communication

use iced::{
    canvas::{self, Cache, Canvas, Cursor, Frame, Geometry},
    executor,
    slider::{self, Slider},
    time, Align, Application, Column, Command, Container, Element, Length, Point, Rectangle, Row,
    Settings, Size, Subscription, Text,
};

use std::thread;
use std::time::{Duration, Instant};

pub fn main() -> iced::Result {
    UI::run(Settings::default())
}

struct UI {
    backend: util::ThreadChannel<simulation::Message>,
    cell_grid: CellGrid,
    target_refresh_rate: u64,
    controls: Controls,
}

// Types of messages that can be sent between UI functions
#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
    EvolutionRateChange(f64),
}

impl Application for UI {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (UI, Command<Self::Message>) {
        let cell_size = 8;
        let grid_size = 96;
        let target_refresh_rate = 60; // Please don't set this to 0
        let evolution_rate = 25; // evolutions/(100s)
        let show_grid_lines = true;
        let grid_line_width = 2.0;
        let controls = Controls {
            evolution_rate_slider: slider::State::new(),
            evolution_rate,
            show_grid_lines: true,
        };
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
            cell_grid: CellGrid::new(cell_size, grid_size, show_grid_lines, grid_line_width),
            target_refresh_rate,
            controls,
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
                                self.cell_grid.cells[position.y][position.x].state = state;
                            }
                        }
                        _ => (),
                    }
                }
            }
            Message::EvolutionRateChange(rate) => {
                self.controls.evolution_rate = rate as u128;
                self.backend
                    .send(simulation::Message::EvolutionRateChange(rate as u128));
            }
        }

        // Async command thingy. No touchy.
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        let canvas = Canvas::new(&self.cell_grid)
            .width(Length::Fill)
            .height(Length::Fill);

        let slider_width = self.cell_grid.cell_size * self.cell_grid.grid_size;

        let controls = self.controls.view(slider_width as u16);
        let content = Column::new().push(canvas).push(controls);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .style(style::Container)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_micros(1_000_000 / self.target_refresh_rate)).map(Message::Tick)
    }
}

struct CellGrid {
    cell_size: usize, // Edge length of cell in pixels
    grid_size: usize, // Edge length of grid in cells
    cells: Vec<Vec<Cell>>,
    frame_content: Cache,
    show_grid_lines: bool,
    line_width: f32,
}

impl CellGrid {
    fn new(cell_size: usize, grid_size: usize, show_grid_lines: bool, line_width: f32) -> Self {
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
            show_grid_lines,
            line_width,
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

            if self.show_grid_lines {
                for line in 0..=self.grid_size {
                    let mut vertical_top_left =
                        Point::from(Position { x: line, y: 0 } * self.cell_size);
                    let mut horizontal_top_left =
                        Point::from(Position { x: 0, y: line } * self.cell_size);

                    let size = (self.grid_size * self.cell_size) as f32 + self.line_width;
                    let vertical_size = Size {
                        width: self.line_width,
                        height: size,
                    };
                    let horizontal_size = Size {
                        width: size,
                        height: self.line_width,
                    };

                    frame.fill_rectangle(vertical_top_left, vertical_size, style::GRID_LINE);
                    frame.fill_rectangle(horizontal_top_left, horizontal_size, style::GRID_LINE);
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

        let color = match self.state {
            CellState::Alive => style::LIVE_CELL,
            CellState::Dead => style::DEAD_CELL,
        };

        frame.fill_rectangle(top_left, size, color);
    }
}

#[derive(Default)]
struct Controls {
    evolution_rate_slider: slider::State,
    evolution_rate: u128,
    show_grid_lines: bool,
    // Pause button
    // Skip button
    // Add x random cells
    // Toggle grid button
    // Evolution rate slider
}

impl Controls {
    fn view(&mut self, slider_width: u16) -> Element<Message> {
        // let speed_controls = Row::new()
        //     .width(Length::Fill)
        //     .align_items(Align::Center)
        //     .spacing(10)
        //     .push(
        //         Slider::new(
        //             &mut self.evolution_rate_slider,
        //             1.0..=200.0,
        //             (self.evolution_rate) as f64,
        //             Message::EvolutionRateChange,
        //         )
        //         .style(style::Slider),
        //     )
        //     .push(
        //         Text::new(format!(
        //             "Evolution rate: {}/s",
        //             (self.evolution_rate as f64) / 10.0
        //         ))
        //         .size(16),
        //     );
        let speed_slider = Row::new()
            .width(Length::Units(slider_width))
            .align_items(Align::Center)
            .spacing(10)
            .push(
                Slider::new(
                    &mut self.evolution_rate_slider,
                    1.0..=200.0,
                    (self.evolution_rate) as f64,
                    Message::EvolutionRateChange,
                )
                .style(style::Slider),
            );
        let text = Text::new(format!(
            "Evolution rate: {}/s",
            (self.evolution_rate as f64) / 10.0
        ))
        .size(16);

        Row::new()
            .width(Length::Shrink)
            .padding(0)
            .spacing(20)
            .align_items(Align::Center)
            .push(speed_slider)
            .push(text)
            .into()
    }
}

struct Statistics {
    // FPS
// Cell count
// Evolution generation
}
