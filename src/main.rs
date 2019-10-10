struct Point { x: u16, y: u16 }

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

enum State {
    Empty,
    Snake,
    Apple,
}

struct BoardPiece {
    state: State,
    position: u16,
}


fn main() {
    // Game component setup.
    let game_size = 30usize;
    let mut board = vec![vec![BoardPiece { state: State::Empty, position: 0 }; game_size];
    game_size];
    let mut direction = Direction::Down;
    let mut snake_length = 3u16;
    let mut head_pos = Point{x: 0, y: 0};
    let mut speed = 0f32;

    initialize_game(&mut board, &mut direction, &mut head_pos);
    start_screen(&game_size);
    loop {
        print_board(&board);
        check_input(&mut direction);
        move_snake(&mut board, &mut snake_length, &mut head_pos);
        wait(&mut speed);
    }
}

fn initialize_game(board: &mut Vec<Vec<BoardSize>>, direction: &mut Direction, head_pos: &mut
Point) {
    spawn_apple(&mut board);
}

spawn_apple(&mut board);

start_screen(&game_size);

print_board(&board);

check_input(&mut direction);

move_snake(&mut board, &mut snake_length, &mut head_pos);

wait(&mut speed);