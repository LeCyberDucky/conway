use crate::util;

use iced::Point;

use rand::Rng;
// use num_traits::Num;

use std::ops::Mul;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl From<Position> for Point {
    fn from(position: Position) -> Self {
        Point {
            x: position.x as f32,
            y: position.y as f32,
        }
    }
}

impl Mul<usize> for Position {
    type Output = Self;

    fn mul(self, rhs: usize) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CellState {
    Alive,
    Dead,
}

#[derive(Debug)]
pub struct Cell {
    pub position: Position, // Top left corner position
    pub state: CellState,
}

impl Cell {
    pub fn new(state: CellState, position: Position) -> Cell {
        Cell { state, position }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    CellTransitions(Vec<(Position, CellState)>),
    EvolutionRateChange(u128),
}

pub struct Simulation {
    grid_size: usize,
    cell_grid: Vec<Vec<Cell>>,
    evolution_rate: u128,
    evolution_count: u128,
    ui: util::ThreadChannel<Message>,
    clock: Instant,
    frame_count: u128,
    target_refresh_rate: u128,
}

impl Simulation {
    pub fn new(
        ui: util::ThreadChannel<Message>,
        grid_size: usize,
        target_refresh_rate: u64,
        evolution_rate: u128,
    ) -> Simulation {
        let mut cell_grid: Vec<Vec<Cell>> = (0..grid_size)
            .map(|y| {
                (0..grid_size)
                    .map(|x| Cell::new(CellState::Dead, Position { x, y }))
                    .collect()
            })
            .collect();

        // Randomly place a number of living cells on the grid
        let living_cell_percent = 50;
        let mut rng = rand::thread_rng();
        let mut live_cells: Vec<Position> = (0..((grid_size * grid_size * living_cell_percent)
            / 100))
            .map(|_| Position {
                x: rng.gen_range(0..grid_size),
                y: rng.gen_range(0..grid_size),
            })
            .collect();

        live_cells.sort();
        live_cells.dedup();

        for Position { x, y } in &live_cells {
            cell_grid[*y][*x].state = CellState::Alive;
        }

        // Set live cells in the UI
        let live_cells = live_cells
            .into_iter()
            .map(|position| (position, CellState::Alive))
            .collect();
        ui.send(Message::CellTransitions(live_cells));

        Simulation {
            grid_size,
            cell_grid,
            evolution_rate, // evolutions/(100s)
            evolution_count: 0,
            ui,
            clock: Instant::now(),
            target_refresh_rate: target_refresh_rate.into(),
            frame_count: 0,
        }
    }

    pub fn run(&mut self) {
        // Check for messages
        // Update
        loop {
            // Check for messages
            let ui_messages = self.ui.receive();
            for message in ui_messages {
                match message {
                    Message::EvolutionRateChange(rate) => {
                        self.evolution_rate = rate;
                        self.evolution_count = 0;
                        self.clock = Instant::now();
                        self.frame_count = 0;
                    }
                    _ => (),
                }
            }

            // Advance simulation
            let clock = self.clock.elapsed().as_millis();
            if 10_000 * (self.evolution_count + 1) <= clock * self.evolution_rate {
                let transitions = self.update();
                self.ui.send(Message::CellTransitions(transitions));
                self.evolution_count += 1;
            }
            self.sleep_remaining_frame();
        }
    }

    fn update(&mut self) -> Vec<(Position, CellState)> {
        let transitions: Vec<Vec<(Position, CellState)>> = self
            .cell_grid
            .iter()
            .map(|row| {
                row.iter()
                    .filter_map(|cell| {
                        let mut neighbor_states = vec![]; // Every cell has 8 neighbors
                        for x_offset in -1..=1_isize {
                            for y_offset in -1..=1_isize {
                                let x = (cell.position.x as isize + x_offset)
                                    .rem_euclid(self.grid_size as isize)
                                    as usize;
                                let y = (cell.position.y as isize + y_offset)
                                    .rem_euclid(self.grid_size as isize)
                                    as usize;

                                if cell.position != (Position { x, y }) {
                                    neighbor_states.push(self.cell_grid[y][x].state);
                                }
                            }
                        }

                        let live_neighbor_count = neighbor_states
                            .iter()
                            .filter(|&state| *state == CellState::Alive)
                            .count();

                        let position = cell.position;

                        match cell.state {
                            CellState::Dead if live_neighbor_count == 3 => {
                                Some((position, CellState::Alive))
                            }
                            CellState::Alive => match live_neighbor_count {
                                0..=1 => Some((position, CellState::Dead)),
                                2..=3 => None,
                                _ => Some((position, CellState::Dead)),
                            },
                            _ => None,
                        }
                    })
                    .collect()
            })
            .collect();

        let transitions: Vec<(Position, CellState)> = transitions.into_iter().flatten().collect();

        for (Position { x, y }, state) in &transitions {
            self.cell_grid[*y][*x].state = *state;
        }

        transitions
    }

    fn sleep_remaining_frame(&mut self) {
        self.frame_count += 1;
        let time_delta = ((self.frame_count * 1000) / self.target_refresh_rate) as i128
            - self.clock.elapsed().as_micros() as i128;

        if time_delta > 0 {
            thread::sleep(Duration::from_micros(time_delta as u64));
        }
    }
}
