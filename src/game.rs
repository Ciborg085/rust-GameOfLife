use std::{io::{self, Write}, sync::atomic::AtomicBool};
use crossterm::{cursor, style::{self, Color, ResetColor, SetBackgroundColor, Stylize}, terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType, SetSize}, ExecutableCommand, QueueableCommand};
use crossterm::cursor::{Hide, Show};
use crossterm::event::{Event,KeyCode,KeyEvent,KeyModifiers,read};
use std::thread;
use std::time;
use rand::prelude::*;
use std::sync::Arc;
use std::sync::atomic::Ordering;

pub struct Game {
    stdout: io::Stdout,
    original_terminal_size: (u16,u16),
    height: u16,
    width: u16,
    list_cells: Vec<Cell>,
}

#[derive(Clone, Copy)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

impl Game {
    pub fn new(stdout: io::Stdout, width: u16, height: u16) -> Self {
        let size = size().unwrap();
        let mut rng = rand::thread_rng();
        let mut random_number: f32 = 0.0;


        //Generate random grid
        let cells: Vec<Cell> = (0..width * height)
            .map(|i| {
                random_number = rng.gen();
                if random_number > 0.5 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        // Generate grid
        //let cells: Vec<Cell> = (0..width * height)
        //    .map(|i| {
        //        if i % 2 == 0 || i % 7 == 0 {
        //            Cell::Alive
        //        } else {
        //            Cell::Dead
        //        }
        //    })
        //    .collect();

        Self {
            height,
            width,
            stdout,
            original_terminal_size: size,
            list_cells: cells,
        }
    }

    pub fn run(&mut self) {


        loop {
            self.prepare_ui();
            self.render();
            self.tick();
            thread::sleep(time::Duration::from_secs(1));
            self.reset_ui();
        }
    }

    fn prepare_ui(&mut self) {
        enable_raw_mode().unwrap();
        self.stdout
            .execute(SetSize(self.width+3,self.height+3)).unwrap()
            .execute(Clear(ClearType::All)).unwrap()
            .execute(SetBackgroundColor(Color::Black)).unwrap()
            .execute(Hide).unwrap();
    }

    
    fn get_index(&self, row: u16, collumn: u16) -> usize {
        (row * self.width +  collumn) as usize
    }

    
    fn live_neighbor_count(&self, row: u16, column: u16) -> u8 {
        let mut count: u8= 0;
        
        for delta_row in [self.height -1, 0, 1].iter().cloned() {
            for delta_col in [self.width -1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let cell_idx = self.get_index(neighbor_row, neighbor_col);
                count += self.list_cells[cell_idx] as u8;
            }
        }
        count
    }

    pub fn tick(&mut self) {
        let mut next = self.list_cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.list_cells[idx];
                let live_neighbor = self.live_neighbor_count(row, col);

                let next_cell = match (cell, live_neighbor) {
                    // Rule 1 - Any live cell with fewer than two live neighbours dies, as if by underpopulation.
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    // Rule 2 - Any live cell with two or three live neighbours lives on to the next generation
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    // Rule 3 - Any live cell with more than three live neighbours dies, as if by overpopulation.
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    // Rule 4 - Any dead cell with exactly three live neighbours becomes a live cell, as if by reproduction.
                    (Cell::Dead, 3) => Cell::Alive,
                    // Default
                    (status,_) => status,
                };
                next[idx] = next_cell;
            }
        }

        self.list_cells = next;
    }

    fn render(&mut self) {
        //self.draw_borders();
        self.draw_background();
        self.draw_cells();
        self.stdout.flush();
    }

    //fn draw_borders(&mut self) {
    //    for row in 0..self.height {
    //        for col in 0..self.width {
    //            if (row == 0 || row == self.height -1 ) || (col == 0 || col == self.width -1) {
    //                self.stdout
    //                    .queue(cursor::MoveTo(row,col)).unwrap()
    //                    .queue(style::PrintStyledContent("█".magenta())).unwrap();
    //            }
    //        }
    //    }
    //    //self.stdout.flush();
    //}

    fn draw_background(&mut self) {
        self.stdout.queue(SetBackgroundColor(Color::Black)).unwrap();
    }

    fn draw_cells(&mut self) {
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.list_cells[idx];
                self.stdout
                    .queue(cursor::MoveTo(row,col)).unwrap()
                    .queue(
                        match cell {
                            Cell::Alive => style::PrintStyledContent("█".magenta()),
                            Cell::Dead => style::PrintStyledContent("█".black())
                        }
                    ).unwrap();
            }
        }
        //self.stdout.flush();
    }

    fn reset_ui(&mut self) {
        let (cols, rows) = self.original_terminal_size;
        self.stdout
            .execute(SetSize(cols, rows)).unwrap()
            .execute(Clear(ClearType::All)).unwrap()
            .execute(Show).unwrap()
            .execute(ResetColor).unwrap();
        disable_raw_mode().unwrap();
    }


}
