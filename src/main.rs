// Could use significant improvements. Most things should be moved out of main. Does not work
// well in a TTY. I'm moving on to my next project for now until I have learned a little more.
// May end up rewriting completely.

extern crate rand;
extern crate termion;

use rand::rngs::ThreadRng;
use std::io::{stdout, Write, Stdout};
use termion::{clear, async_stdin, AsyncReader, terminal_size};
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
    let term_size = terminal_size().unwrap().1;
    // Determine game size based on screen size and make it close to a rectangle.
    let game_size = Point{
        x: term_size as usize * 2 - 5, // x should be odd
        y: term_size as usize - 5 // Buffer compensates for borders and makes it look cleaner
    };

    let mut board =
        vec![vec![BoardPiece { state: State::Empty, position: 0 }; game_size.x]; game_size.y];

    let mut direction = Direction::Down;;
    let mut snake_length = 3; // default snake length

    // Non-blocking key press reader.
    let mut stdin = async_stdin().keys();
    let mut stdout = stdout().into_raw_mode().unwrap();

    // Adds top, bottom, and side borders to game.
    add_border(&mut board);
    // Places the snake and apple on the board and sets the snake's initial movement direction.
    initialize_game(&mut board, &mut direction, &snake_length);
    // Simple intro screen.
    start_screen(&mut stdin, &mut stdout);
    // Flag to break the main loop. Initially used an exit statement, but this put the terminal in
    // and unusable state at exit.
    let mut end_game;
    'main: loop {
        print_board(&board, &mut stdout);
        // Checks for player input and sets the direction accordingly. Arrows or WASD.
        check_input(&mut stdin, &mut direction);
        // Returns true if player hits a wall or the end of the snake.
        end_game = move_snake(&mut board, &mut snake_length, &direction);
        if end_game {
            // Displays the ending score and returns true if the player wants to play again.
            let play_again = game_over(&mut stdin, &mut stdout, &snake_length);
            if !play_again {
                break 'main;
            } else {
                // Changes everything but the borders into Empty pieces.
                clear_board(&mut board);
                // Reinitialize and start fresh.
                snake_length = 3;
                initialize_game(&mut board, &mut direction, &snake_length);
                start_screen(&mut stdin, &mut stdout);
            }
        }
        wait();
    }
}


/// Replaces all non-border pieces with Empty BoardPieces.
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
    let x_bound = board[1].len() / 2;
    // The x value of the snake will only ever be odd because of the space skipping.
    let y_bound = board.len() / 2;
    let head_pos = Point {
        x: gen_odd_number(&mut rng, x_bound - *snake_length, x_bound + *snake_length),
        y: rng.gen_range(y_bound - *snake_length, y_bound + *snake_length),
    };
    // Change to BoardPiece at head_pos to a snake piece.
    board[head_pos.y][head_pos.x] = BoardPiece {
        state: State::Snake,
        position: 1,
    };
    // Generate a random direction for the snake to travel.
    *direction = rng.gen();
    // Generate the next two pieces. Could possibly be made able to generate n snake pieces
    // instead of fixed.
    for i in 1..*snake_length {
        let mut snake_part_pos = Point {
            x: head_pos.x,
            y: head_pos.y,
        };
        match *direction {
            Direction::Up => snake_part_pos.y += i,
            Direction::Down => snake_part_pos.y -= i,
            Direction::Left => snake_part_pos.x += (i * 2),
            Direction::Right => snake_part_pos.x -= (i * 2),
        }
        board[snake_part_pos.y][snake_part_pos.x] = BoardPiece {
            state: State::Snake,
            position: i + 1,
        };
    }
    spawn_apple(&mut *board, &mut rng);
}

fn gen_odd_number(rng: &mut impl Rng, lower: usize, upper: usize) -> usize {
    let mut x = 2usize;
    while x % 2 == 0 {
        // x and y are generated with a buffer so that the snake doesn't end up in the wall.
        x = rng.gen_range(lower, upper)
    }
    x
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
        let mut x = 2;
        while x % 2 == 0 {
            x = rng.gen_range(0, board[0].len() - 2)
        }
        let y = rng.gen_range(0, board.len());
        if board[y][x].state == State::Empty {
            board[y][x].state = State::Apple;
            break;
        }
    }
}

fn start_screen(input: &mut Keys<AsyncReader>, terminal: &mut RawTerminal<Stdout>) {
    write!(&mut *terminal, "{}", "\u{256D}").unwrap();
    let mut hello: Vec<char> = "Welcome to Terminal Snake! Press Space to start.".chars().collect();
    for _ in 0..hello.len() {
        write!(&mut *terminal, "{}", "\u{2500}").unwrap();
    }
    write!(&mut *terminal, "{}", "\u{256E}\r\n").unwrap();
    write!(&mut *terminal, "{}", "\u{2502}").unwrap();
    for c in hello.iter() {
        write!(&mut *terminal, "{}", c).unwrap();
    }
    write!(&mut *terminal, "{}", "\u{2502}\r\n").unwrap();
    write!(&mut *terminal, "{}", "\u{2570}").unwrap();
    for _ in 0..hello.len() {
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
            if head.x == 1 {
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
    loop {
        match input.next() {
            Some(Ok(Key::Char(' '))) => return true,
            Some(Ok(Key::Esc)) => return false,
            _ => {},
        }
    }
}