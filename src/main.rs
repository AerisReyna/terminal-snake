extern crate rand;
extern crate termion;

use rand::rngs::ThreadRng;
use std::io::{stdin, stdout, Write, Stdout};
use termion::clear;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::input::TermRead;
use termion::event::Key;
use rand::Rng;
use rand::distributions::{Distribution, Standard};
use std::process::exit;


struct Point { x: usize, y: usize }

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.gen_range(0, 4) {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            _ => Direction::Right,
        }
    }
}

#[derive(Clone, PartialEq)]
enum State {
    Empty,
    Snake,
    Apple,
    Border(String),
}

#[derive(Clone)]
struct BoardPiece {
    state: State,
    position: usize,
}


fn main() {
    // Game component setup.
    let game_size = 30usize; // If this is changed, be sure to check length of hello
                                      // message in start screen.
    let mut board = vec![vec![BoardPiece { state: State::Empty, position: 0 }; game_size * 2 + 1];
                         game_size];
    let mut direction = Direction::Down;
    let mut snake_length = 3usize;
    let mut head_pos = Point { x: 0, y: 0 };
    let mut speed = 0f32;

    let mut stdout = stdout().into_raw_mode().unwrap();


    initialize_game(&mut board, &mut direction, &mut head_pos, &snake_length);
    start_screen(&mut stdout, &game_size);
    'main: loop {
        print_board(&board, &mut stdout);
        // This is the only way to exit the program and retain the ability to type to the terminal.
        match check_input(&mut direction) {
            Ok(_) => {},
            Err(_) => break 'main,
        }
        //move_snake(&mut board, &mut snake_length, &mut head_pos);
        //wait(&mut speed);
    }
}

fn initialize_game(board: &mut Vec<Vec<BoardPiece>>, direction: &mut Direction, head_pos: &mut
Point, snake_length: &usize) {
    // Initial random placement of snake and first apple.
    let mut rng = rand::thread_rng();
    let mut x = 1usize;
    // x coordinate must be an even number. The x skips inserts an extra blank space between
    // the snake pieces so that the terminal display does not appear horizontally.
    while x % 2 != 0 {
        x = rng.gen_range(*snake_length * 2, board[0].len() - *snake_length * 2)
    }
    head_pos.x = x;
    head_pos.y = rng.gen_range(*snake_length, board.len() - *snake_length);
    board[head_pos.y][head_pos.x].state = State::Snake;
    *direction = rng.gen();
    match *direction {
        Direction::Down => {
            board[head_pos.y + 1][head_pos.x].state = State::Snake;
            board[head_pos.y + 2][head_pos.x].state = State::Snake
        }
        Direction::Up => {
            board[head_pos.y - 1][head_pos.x].state = State::Snake;
            board[head_pos.y - 2][head_pos.x].state = State::Snake
        }
        Direction::Right => {
            board[head_pos.y][head_pos.x + 2].state = State::Snake;
            board[head_pos.y][head_pos.x + 4].state = State::Snake
        }
        Direction::Left => {
            board[head_pos.y][head_pos.x - 2].state = State::Snake;
            board[head_pos.y][head_pos.x - 4].state = State::Snake
        }
    }
    spawn_apple(&mut *board, &mut rng, &snake_length);

    // Insertion of top and bottom borders.
    let mut top_border = Vec::new();
    top_border.push(BoardPiece { state: State::Border(String::from("\u{256D}")), position: 0 });
    for _ in 0..board[0].len() {
        top_border.push(BoardPiece{state: State::Border(String::from("\u{2500}")), position: 0});
    }
    top_border.push(BoardPiece { state: State::Border(String::from("\u{256E}")), position: 0 });
    board.insert(0, top_border);

    let mut bottom_border = Vec::new();
    bottom_border.push(BoardPiece { state: State::Border(String::from("\u{2570}")), position: 0 });
    for _ in 0..board[0].len() - 2 {
        bottom_border.push(BoardPiece{state: State::Border(String::from("\u{2500}")), position: 0});
    }
    bottom_border.push(BoardPiece { state: State::Border(String::from("\u{256F}")), position: 0 });
    board.push(bottom_border);

    // Insertion of side bars.
    for i in 1..board.len() - 1 {
        board[i].insert(0, BoardPiece{state: State::Border(String::from("\u{2502}")), position: 0});
        board[i].push(BoardPiece{state: State::Border(String::from("\u{2502}")), position: 0});
    }
}

fn spawn_apple(board: &mut Vec<Vec<BoardPiece>>, rng: &mut ThreadRng, snake_length: &usize) {
    loop {
        let mut x = 1;
        while x % 2 != 0 {
            x = rng.gen_range(*snake_length * 2, board[0].len() - *snake_length * 2)
        }
        let y = rng.gen_range(0, board.len());
        if board[y][x].state == State::Empty {
            board[y][x].state = State::Apple;
            break;
        }
    }
}

fn start_screen(terminal: &mut RawTerminal<Stdout>, size: &usize) {
    write!(&mut *terminal, "{}", "\u{256D}");
    for _ in 1..size * 2 - 1 {
        write!(&mut *terminal, "{}", "\u{2500}");
    }
    write!(&mut *terminal, "{}", "\u{256E}\r\n");
    write!(&mut *terminal, "{}", "\u{2502}");
    let mut hello: Vec<char> = "Welcome to Terminal Snake! Press Space to start.".chars().collect();
    for _ in 0..(size * 2 - hello.len()) / 2 {
        hello.insert(0, ' ');
    }
    for _ in 0..(size * 2 - hello.len()) / 2 + 1 {
        hello.push(' ');
    }
    for c in hello.iter() {
        write!(&mut *terminal, "{}", c);
    }
    write!(&mut *terminal, "{}", "\u{2502}\r\n");
    write!(&mut *terminal, "{}", "\u{2570}");
    for _ in 1..size * 2 - 1 {
        write!(&mut *terminal, "{}", "\u{2500}");
    }
    write!(&mut *terminal, "{}", "\u{256F}\r\n");
    'a: loop {
        let stdin = stdin();
        for key in stdin.keys() {
            match key.unwrap() {
                Key::Char(' ') => break 'a,
                _ => {},
            }
        }
    }
}

fn print_board(board: &Vec<Vec<BoardPiece>>, stdout: &mut RawTerminal<Stdout>) {
    write!(&mut *stdout, "{}", clear::All);
    for i in 0..board.len() {
        for j in 0..board[i].len() {
            match &board[i][j].state {
                State::Empty => write!(&mut *stdout, "{}", ' ').unwrap(),
                State::Apple => write!(&mut *stdout, "{}", '@').unwrap(),
                State::Snake => write!(&mut *stdout, "{}", '*').unwrap(),
                State::Border(all) => write!(&mut *stdout, "{}", &all).unwrap(),
            }
        }
        write!(&mut *stdout, "{}{}", '\r', '\n').unwrap()
    }
}

fn check_input(direction: &mut Direction) -> Result<(), i32>{
    let stdin = stdin();
    for key in stdin.keys() {
        match key.unwrap() {
            Key::Char('w') => *direction = Direction::Up,
            Key::Char('a') => *direction = Direction::Left,
            Key::Char('s') => *direction = Direction::Down,
            Key::Char('d') => *direction = Direction::Right,
            Key::Esc => return Err(1),
            _ => {},
        }
    }
    Ok(())
}

fn move_snake(board: &mut Vec<Vec<BoardPiece>>, snake_length: &mut usize, head_pos: &mut Point) {}

fn wait(speed: &mut f32) {}