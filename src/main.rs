extern crate rand;
extern crate termion;

use rand::rngs::ThreadRng;
use std::io::{stdout, Write, Stdout};
use termion::{clear, async_stdin, AsyncReader};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::input::{TermRead, Keys};
use termion::event::Key;
use rand::Rng;
use rand::distributions::{Distribution, Standard};
use std::{thread, time};


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
    let mut direction;
    let mut snake_length;

    let mut stdin = async_stdin().keys();
    let mut stdout = stdout().into_raw_mode().unwrap();


    initialize_game(&mut board, &mut direction, &snake_length);
    add_border(&mut board);
    start_screen(&mut stdin, &mut stdout, &game_size);
    let mut end_game;
    'main: loop {
        print_board(&board, &mut stdout);
        // This is the only way to exit the program and retain the ability to type to the terminal.
        check_input(&mut stdin, &mut direction);
        end_game = move_snake(&mut board, &mut snake_length, &direction);
        if end_game {
            let play_again = game_over(&mut stdin, &mut stdout, &snake_length);
            if !play_again {
                break 'main;
            } else {
                clear_board(&mut board);
                snake_length = 3;
                initialize_game(&mut board, &mut direction, &snake_length);
                start_screen(&mut stdin, &mut stdout, &game_size);
            }
        }
        wait();
    }
}

fn clear_board(board: &mut Vec<Vec<BoardPiece>>) {
    for i in 0..board.len() {
        for j in 0..board[i].len() {
            match board[i][j].state {
                State::Border(_) => {},
                _ => board[i][j] = BoardPiece{state: State::Empty, position: 0},
            }

        }
    }
}

fn initialize_game(board: &mut Vec<Vec<BoardPiece>>, direction: &mut Direction, snake_length: &usize) {
    // Initial random placement of snake and first apple.
    let mut rng = rand::thread_rng();
    let mut x = 1usize;
    // x coordinate must be an even number. The x skips inserts an extra blank space between
    // the snake pieces so that the terminal display does not appear horizontally.
    while x % 2 != 0 {
        // x and y are generated with a buffer so that the snake doesn't end up in the wall.
        x = rng.gen_range(*snake_length * 2, board[0].len() - *snake_length * 2)
    }
    let mut head_pos = Point{x: 0, y: 0};
    head_pos.x = x;
    head_pos.y = rng.gen_range(*snake_length, board.len() - *snake_length);
    board[head_pos.y][head_pos.x].state = State::Snake;
    board[head_pos.y][head_pos.x].position = 1;
    *direction = rng.gen();
    let mut second = Point{x: head_pos.x, y: head_pos.y};
    let mut third = Point{x: head_pos.x, y: head_pos.y};
    // Position pieces of the snake in the opposite direction that it is facing.
    match *direction {
        Direction::Down => {
            second.y = head_pos.y - 1;
            third.y = head_pos.y - 2;
        }
        Direction::Up => {
            second.y = head_pos.y + 1;
            third.y = head_pos.y + 2;
        }
        Direction::Right => {
            second.x = head_pos.x - 2;
            third.x = head_pos.x - 4;
        }
        Direction::Left => {
            second.x = head_pos.x + 2;
            third.x = head_pos.x + 4;
        }
    }
    board[second.y][second.x].state = State::Snake;
    board[second.y][second.x].position = 2;
    board[third.y][third.x].state = State::Snake;
    board[third.y][third.x].position = 3;
    spawn_apple(&mut *board, &mut rng);
}

fn add_border(board: &mut Vec<Vec<BoardPiece>>) {
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

fn spawn_apple(board: &mut Vec<Vec<BoardPiece>>, rng: &mut ThreadRng) {
    loop {
        let mut x = 1;
        while x % 2 != 0 {
            x = rng.gen_range(1, board[0].len() - 2)
        }
        let y = rng.gen_range(0, board.len());
        if board[y][x].state == State::Empty {
            board[y][x].state = State::Apple;
            break;
        }
    }
}

fn start_screen(input: &mut Keys<AsyncReader>, terminal: &mut RawTerminal<Stdout>, size: &usize) {
    write!(&mut *terminal, "{}", "\u{256D}").unwrap();
    for _ in 1..size * 2 - 1 {
        write!(&mut *terminal, "{}", "\u{2500}").unwrap();
    }
    write!(&mut *terminal, "{}", "\u{256E}\r\n").unwrap();
    write!(&mut *terminal, "{}", "\u{2502}").unwrap();
    let mut hello: Vec<char> = "Welcome to Terminal Snake! Press Space to start.".chars().collect();
    for _ in 0..(size * 2 - hello.len()) / 2 {
        hello.insert(0, ' ');
    }
    for _ in 0..(size * 2 - hello.len()) / 2 + 1 {
        hello.push(' ');
    }
    for c in hello.iter() {
        write!(&mut *terminal, "{}", c).unwrap();
    }
    write!(&mut *terminal, "{}", "\u{2502}\r\n").unwrap();
    write!(&mut *terminal, "{}", "\u{2570}").unwrap();
    for _ in 1..size * 2 - 1 {
        write!(&mut *terminal, "{}", "\u{2500}").unwrap();
    }
    write!(&mut *terminal, "{}", "\u{256F}\r\n").unwrap();
    'a: loop {
        match input.next() {
            Some(Ok(Key::Char(' '))) => break 'a,
            _ => {},
        }
    }
}

fn print_board(board: &Vec<Vec<BoardPiece>>, stdout: &mut RawTerminal<Stdout>) {
    write!(&mut *stdout, "{}", clear::All).unwrap();
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

fn check_input(stdin: &mut Keys<AsyncReader>, direction: &mut Direction) {
    match stdin.next() {
        Some(Ok(Key::Up)) | Some(Ok(Key::Char('w'))) => *direction = Direction::Up,
        Some(Ok(Key::Left)) | Some(Ok(Key::Char('a'))) => *direction = Direction::Left,
        Some(Ok(Key::Down)) | Some(Ok(Key::Char('s'))) => *direction = Direction::Down,
        Some(Ok(Key::Right)) | Some(Ok(Key::Char('d'))) => *direction = Direction::Right,
        _ => {},
    }
}

fn move_snake(board: &mut Vec<Vec<BoardPiece>>, snake_length: &mut usize, direction: &Direction)
    -> bool {
    let mut head = Point{x: 0, y: 0};
    for i in 0..board.len() {
        for j in 0..board[i].len() {
            let current = &mut board[i][j];
            if current.state == State::Snake {
                if current.position == 1 {
                    head = Point{x: j, y: i};
                }
                current.position += 1;
                if current.position > *snake_length {
                    current.state = State::Empty;
                    current.position = 0;
                }
            }
        }
    }
    match *direction {
        Direction::Up => {
            if head.y == 0 {
                return true;
            }
            match board[head.y - 1][head.x].state {
                State::Border(_) | State::Snake => return true,
                State::Apple => {
                    board[head.y - 1][head.x] = BoardPiece {
                        state: State::Snake,
                        position: 1,
                    };
                    *snake_length += 1;
                    let mut rng = rand::thread_rng();
                    spawn_apple(&mut *board, &mut rng);
                },
                State::Empty => {
                    board[head.y - 1][head.x] = BoardPiece {
                        state: State::Snake,
                        position: 1,
                    };
                },
            }
        },
        Direction::Down => {
            match board[head.y + 1][head.x].state {
                State::Border(_) | State::Snake => return true,
                State::Apple => {
                    board[head.y + 1][head.x] = BoardPiece {
                        state: State::Snake,
                        position: 1,
                    };
                    *snake_length += 1;
                    let mut rng = rand::thread_rng();
                    spawn_apple(&mut *board, &mut rng);
                },
                State::Empty => {
                    board[head.y + 1][head.x] = BoardPiece {
                        state: State::Snake,
                        position: 1,
                    };
                },
            }
        }
        Direction::Left => {
            if head.x == 0 {
                return true;
            }
            match board[head.y][head.x - 2].state {
                State::Border(_) | State::Snake => return true,
                State::Apple => {
                    board[head.y][head.x - 2] = BoardPiece {
                        state: State::Snake,
                        position: 1,
                    };
                    *snake_length += 1;
                    let mut rng = rand::thread_rng();
                    spawn_apple(&mut *board, &mut rng);
                },
                State::Empty => {
                    board[head.y][head.x - 2] = BoardPiece {
                        state: State::Snake,
                        position: 1,
                    };
                },
            }
        }
        Direction::Right => {
            if head.x + 2 > board[0].len() - 1 {
                return true;
            }
            match board[head.y][head.x + 2].state {
                State::Border(_) | State::Snake => return true,
                State::Apple => {
                    board[head.y][head.x + 2] = BoardPiece {
                        state: State::Snake,
                        position: 1,
                    };
                    *snake_length += 1;
                    let mut rng = rand::thread_rng();
                    spawn_apple(&mut *board, &mut rng);
                },
                State::Empty => {
                    board[head.y][head.x + 2] = BoardPiece {
                        state: State::Snake,
                        position: 1,
                    };
                }
            }
        }
    }
    false
}

fn wait() {
    thread::sleep(time::Duration::from_millis(100));
}

fn game_over(input: &mut Keys<AsyncReader>, terminal: &mut RawTerminal<Stdout>, score: &usize) ->
                                                                                             bool {
    let message: Vec<char> =
        format!("You scored {} points! Would you like to play again? Space to play, Esc to quit\
        .", &score)
            .chars()
            .collect();
    write!(&mut *terminal, "{}", "\u{256D}").unwrap();
    for _ in 0..message.len() {
        write!(&mut *terminal, "{}", "\u{2500}").unwrap();
    }
    write!(&mut *terminal, "{}", "\u{256E}\r\n").unwrap();
    write!(&mut *terminal, "{}", "\u{2502}").unwrap();
    for c in message.iter() {
        write!(&mut *terminal, "{}", c).unwrap();
    }
    write!(&mut *terminal, "{}", "\u{2502}\r\n").unwrap();
    write!(&mut *terminal, "{}", "\u{2570}").unwrap();
    for _ in 0..message.len() {
        write!(&mut *terminal, "{}", "\u{2500}").unwrap();
    }
    write!(&mut *terminal, "{}", "\u{256F}\r\n").unwrap();
    'a: loop {
        match input.next() {
            Some(Ok(Key::Char(' '))) => return true,
            Some(Ok(Key::Esc)) => return false,
            _ => {},
        }
    }
}