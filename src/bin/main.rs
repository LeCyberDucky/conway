mod simulation;
use simulation::{Cell, CellState, Position};
mod style;
mod util; // Contains channels for inter-thread communication

use iced::{Align, Application, Column, Command, Container, Element, HorizontalAlignment, Length, Point, Rectangle, Row, Settings, Size, Space, Subscription, Text, button::{self, Button}, canvas::{self, Cache, Canvas, Cursor, Frame, Geometry}, executor, slider::{self, Slider}, text_input::{self, TextInput}, time};

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
    statistics: Statistics,
}

// Types of messages that can be sent between UI functions
#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
    EvolutionRateChange(f64),
    TogglePlay,
    Evolve(usize),
    SetEvolveCount(usize, String),
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
        let is_paused = true;
        let evolve_count = 1;
        let controls = Controls {
            evolution_rate_slider: slider::State::new(),
            evolution_rate,
            show_grid_lines: true,
            is_paused,
            toggle_play_button: button::State::new(),
            evolve_button: button::State::new(),
            evolve_count,
            evolve_input_field: text_input::State::new(),
            evolve_input_text: evolve_count.to_string(),
        };
        let statistics = Statistics {
            cell_count: grid_size * grid_size,
            live_cell_count: 0,
            generation: 0,
        };
        let (ui, backend) = util::ThreadChannel::new_pair();

        thread::Builder::new()
            .name("Game of Life Simulation".to_string())
            .spawn(move || {
                let mut simulation = simulation::Simulation::new(
                    ui,
                    grid_size,
                    target_refresh_rate,
                    evolution_rate,
                    is_paused,
                );
                simulation.run();
            })
            .unwrap(); // Not sure what to do here besides this unwrap, as I'm not the one calling this outer function

        let ui = UI {
            backend,
            cell_grid: CellGrid::new(cell_size, grid_size, show_grid_lines, grid_line_width),
            target_refresh_rate,
            controls,
            statistics,
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

                                match state {
                                    CellState::Alive => self.statistics.live_cell_count += 1,
                                    CellState::Dead => self.statistics.live_cell_count -= 1,
                                }
                            }
                            self.statistics.generation += 1;
                        }
                        _ => (),
                    }
                }
            },
            Message::EvolutionRateChange(rate) => {
                self.controls.evolution_rate = rate as u128;
                self.backend
                    .send(simulation::Message::EvolutionRateChange(rate as u128));
            },
            Message::TogglePlay => {
                self.controls.is_paused = !self.controls.is_paused;
                self.backend.send(simulation::Message::TogglePlay);
            },
            Message::Evolve(generations) => {
                self.backend.send(simulation::Message::Evolve(generations));
            },
            Message::SetEvolveCount(count, text) => {
                self.controls.evolve_count = count;
                self.controls.evolve_input_text = text;
            }
        }

        // Async command thingy. No touchy.
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        let canvas_width = self.cell_grid.cell_size * self.cell_grid.grid_size
            + self.cell_grid.line_width as usize;
        let canvas = Canvas::new(&self.cell_grid)
            .width(Length::Units(canvas_width as u16))
            .height(Length::Units(canvas_width as u16));

        let statistics = self.statistics.view();

        let content = Row::new().spacing(10).push(canvas).push(statistics);
        let (bottom_controls, right_controls) = self.controls.view(canvas_width as u16);
        let content = Column::new().push(content).push(bottom_controls);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
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
                    cell.draw(frame, self.cell_size, self.line_width / 2.0);
                }
            }

            if self.show_grid_lines {
                let line_size = (self.grid_size * self.cell_size) as f32 + self.line_width;

                let vertical_size = Size {
                    width: self.line_width,
                    height: line_size,
                };
                let horizontal_size = Size {
                    width: line_size,
                    height: self.line_width,
                };

                for line in 0..=self.grid_size {
                    let vertical_top_left =
                        Point::from(Position { x: line, y: 0 } * self.cell_size);
                    let horizontal_top_left =
                        Point::from(Position { x: 0, y: line } * self.cell_size);

                    frame.fill_rectangle(vertical_top_left, vertical_size, style::GRID_LINE);
                    frame.fill_rectangle(horizontal_top_left, horizontal_size, style::GRID_LINE);
                }
            }
        });

        vec![frame_conent]
    }
}

impl Cell {
    fn draw(&self, frame: &mut Frame, size: usize, offset: f32) {
        let mut top_left = Point::from(self.position * size);
        top_left.x += offset;
        top_left.y += offset;
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
    is_paused: bool,
    toggle_play_button: button::State,
    evolve_button: button::State,
    evolve_input_field: text_input::State,
    evolve_input_text: String,
    evolve_count: usize,
    // Add x random cells
    // Toggle grid button
    // Click to toggle state of cell
}

impl Controls {
    fn view(&mut self, slider_width: u16) -> (Element<Message>, Element<Message>) {
        let speed_slider = Slider::new(
            &mut self.evolution_rate_slider,
            1.0..=200.0,
            (self.evolution_rate) as f64,
            Message::EvolutionRateChange,
        )
        .style(style::Slider);

        let play_button = Button::new(
            &mut self.toggle_play_button,
            match self.is_paused {
                false => Text::new(format!("Pause")).size(18),
                true => Text::new(format!("Play")).size(18),
            },
        )
        .on_press(Message::TogglePlay).style(style::Button);

        let evolve_button = Button::new(
            &mut self.evolve_button,
            Text::new(format!("Evolve by:")).size(18),
        )
        .on_press(Message::Evolve(self.evolve_count)).style(style::Button);

        let evolve_count = self.evolve_count;

        let evolve_input_field = TextInput::new(
            &mut self.evolve_input_field,
            "Evolve X generations",
            &self.evolve_input_text,
            (move |input| Controls::input_evolve_count(input, evolve_count)),
        ).padding(5).style(style::InputField);

        let evolution_rate = Text::new(format!(
            "Evolutions/second: {}",
            if self.is_paused {
                0.0
            } else {
                (self.evolution_rate as f64) / 10.0
            }
        ))
        .size(18);
        let evolution_rate = Container::new(evolution_rate)
            .padding(5)
            .style(style::TextSnippet);

        let evolution_controls = Row::new()
            .width(Length::Units(slider_width))
            .align_items(Align::Center)
            .spacing(5)
            .push(play_button)
            .push(evolve_button)
            .push(evolve_input_field)
            .push(Space::with_width(Length::Fill))
            .push(evolution_rate);

        let bottom = Column::new()
            .width(Length::Units(slider_width))
            .spacing(5)
            .push(speed_slider)
            .push(evolution_controls)
            .into();

        let side = Column::new().into();

        (bottom, side)
    }

    fn input_evolve_count(input: String, current_number: usize) -> Message {
        let count = input.parse::<usize>();
        match count {
            Ok(number) => Message::SetEvolveCount(number, number.to_string()),
            Err(_) => Message::SetEvolveCount(0, input),
        }
    }
}

struct Statistics {
    cell_count: usize,
    live_cell_count: usize,
    generation: usize,
    // FPS
}

impl Statistics {
    fn view(&mut self) -> Element<Message> {
        let total_cells = self.cell_count;
        let live_cells = self.live_cell_count;
        let dead_cells = total_cells - live_cells;
        let live_cell_percent = (live_cells as f64) / (total_cells as f64) * 100.0;
        let dead_cell_percent = (dead_cells as f64) / (total_cells as f64) * 100.0;

        let statistics = Text::new(format!(
            "Generation: {}\n\nTotal cells: {}\nLive cells: {} ≈ {:.2}%\nDead cells: {} ≈ {:.2}%",
            self.generation,
            total_cells,
            live_cells,
            live_cell_percent,
            dead_cells,
            dead_cell_percent
        ))
        .size(18);

        Container::new(statistics)
            .padding(5)
            .style(style::TextSnippet)
            .into()
    }
}
