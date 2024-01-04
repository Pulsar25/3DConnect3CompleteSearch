use std::collections::HashSet;
use std::fs::OpenOptions;
use std::hash::Hash;
use std::io::{Write, BufWriter};
use std::sync::{Arc, Mutex};
extern crate crossbeam;
use crossbeam::channel::unbounded;
use graphlib::Graph;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use rayon::prelude::*;
use bincode::{serialize_into, deserialize_from};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Pair {
    first: i64,
    second: i64,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
struct Board {
    data: [[[i8; 3]; 3]; 3],
}

#[derive(Hash, Eq, PartialEq, Clone)]
struct Game {
    board: Board,
    player: i8,
}

fn make_new_board() -> Board {
    Board {
        data: [[[0; 3]; 3]; 3],
    }
}

fn switch_player(player: i8) -> i8 {
    if player == 1 {
        2
    } else {
        1
    }
}

fn get_top(g: Board, x: usize, y: usize) -> Option<usize> {
    for (i, &value) in g.data[x][y].iter().enumerate() {
        if value == 0 {
            return Some(i);
        }
    }
    None
}

fn place_new_piece(g: &mut Board, x: usize, y: usize, player: i8) -> Option<usize> {
    let z: Option<usize> = get_top(Board { data: g.data }, x, y);
    if z.is_some() {
        let unwrapped = z.unwrap();
        g.data[x][y][unwrapped] = player;
    }
    z
}

fn get_all_next_states(g: Board, player: i8) -> Vec<Board> {
    let mut output: Vec<Board> = Vec::new();
    for x in 0..3 {
        for y in 0..3 {
            let mut new_board = g.clone();
            let worked = place_new_piece(&mut new_board, x, y, player);
            if worked.is_some() {
                output.push(new_board);
            }
        }
    }
    output
}

fn check_win_direction(
    g: &Game,
    x: i8,
    y: i8,
    z: i8,
    dirx: i8,
    diry: i8,
    dirz: i8,
    left: i8,
    last: i8,
) -> bool {
    if left == 0 {
        return false;
    }
    if x > 3 || x < 0 || y > 3 || y < 0 || z > 3 || z < 0 {
        return false;
    }
    if g.board.data[x as usize][y as usize][z as usize] == 0 {
        return false;
    }
    if g.board.data[x as usize][y as usize][z as usize] == last || left == 3 {
        if left == 1 {
            return true;
        } else {
            if left == 3 {
                return check_win_direction(
                    g,
                    x + dirx,
                    y + diry,
                    z + dirz,
                    dirx,
                    diry,
                    dirz,
                    left - 1,
                    g.board.data[x as usize][y as usize][z as usize],
                );
            } else {
                return check_win_direction(
                    g,
                    x + dirx,
                    y + diry,
                    z + dirz,
                    dirx,
                    diry,
                    dirz,
                    left - 1,
                    last,
                );
            }
        }
    }
    return false;
}

fn _get_num_open_spaces(g: &Game) -> i8 {
    let mut output: i8 = 0;
    for x in 0..3 {
        for y in 0..3 {
            for z in 0..3 {
                if g.board.data[x][y][z] == 0 {
                    output += 1;
                }
            }
        }
    }
    return output;
}

fn is_over(g: &Game) -> bool {
    for y in 0..3 {
        for z in 0..3 {
            if check_win_direction(g, 0, y, z, 1, 0, 0, 3, -1) {
                return true;
            }
        }
    }
    for x in 0..3 {
        for z in 0..3 {
            if check_win_direction(g, x, 0, z, 0, 1, 0, 3, -1) {
                return true;
            }
        }
    }
    for x in 0..3 {
        for y in 0..3 {
            if check_win_direction(g, x, y, 0, 0, 0, 1, 3, -1) {
                return true;
            }
        }
    }
    for x in 0..3 {
        if check_win_direction(g, x, 0, 0, 0, 1, 1, 3, -1) {
            return true;
        }
        if check_win_direction(g, x, 2, 2, 0, -1, -1, 3, -1) {
            return true;
        }
    }
    for y in 0..3 {
        if check_win_direction(g, 0, y, 0, 1, 0, 1, 3, -1) {
            return true;
        }
        if check_win_direction(g, 2, y, 2, -1, 0, -1, 3, -1) {
            return true;
        }
    }
    for z in 0..3 {
        if check_win_direction(g, 0, 0, z, 1, 1, 0, 3, -1) {
            return true;
        }
        if check_win_direction(g, 2, 2, z, -1, -1, 0, 3, -1) {
            return true;
        }
    }
    if check_win_direction(g, 0, 0, 0, 1, 1, 1, 3, -1) {
        return true;
    }
    if check_win_direction(g, 0, 2, 0, 1, -1, 1, 3, -1) {
        return true;
    }
    if check_win_direction(g, 2, 0, 0, -1, 1, 1, 3, -1) {
        return true;
    }
    if check_win_direction(g, 2, 2, 0, -1, -1, 1, 3, -1) {
        return true;
    }
    return false;
}

fn board_to_number(g: Game) -> i64 {
    let mut output: i64 = 0;
    for x in 0..3 {
        for y in 0..3 {
            for z in 0..3 {
                output += (g.board.data[x][y][z] as i64) * 3_i64.pow((x + 3 * y + 9 * z) as u32);
            }
        }
    }
    return output;
}

fn _number_to_board(mut num: i64) -> Game {
    let mut output: Game = Game {
        board: make_new_board(),
        player: 1,
    };
    for power in 0..27 {
        output.board.data[power % 3][power / 3][power / 9] = (num % 3) as i8;
        num -= num % 3;
        num /= 3;
    }
    return output;
}

fn process() {
    let file = File::open("C:\\Users\\evana\\Desktop\\Connect3\\input.txt").unwrap();
    let reader = BufReader::new(file);
    let mut output = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("data.bin")
        .unwrap();
    let (pair_sender, pair_reciever) = unbounded::<(i64, i64)>();
    reader
        .lines()
        .par_bridge() // Parallel processing
        .for_each(|line| {
            if let Ok(line) = line {
                let pair_sender_clone = pair_sender.clone();
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() == 2 {
                    let a : i64 = parts[0].parse().unwrap();
                    let b : i64 = parts[1].parse().unwrap();
                    pair_sender_clone.send((a,b));
                }
            }
        });
    loop {
        match pair_reciever.recv() {
            Ok((a, b)) => {
                serialize_into(&mut output, &(a,b));
            }
            Err(_) => {
                break;
            }
        }
    }   
    println!("test")
}

fn main() {
    process();
}

fn _generate() {
    let seen: Arc<Mutex<HashSet<i64>>> = Arc::new(Mutex::new(HashSet::new()));
    let mut handles = Vec::new();
    let (work_queue_sender, work_queue_receiver) = unbounded::<Game>();
    let (result_queue_sender, result_queue_receiver) = unbounded::<(i64, i64)>();
    let _ = work_queue_sender.send(Game {
        board: make_new_board(),
        player: 1,
    });
    let n_threads = 16;
    for i in 0..n_threads {
        let shared_set_clone = Arc::clone(&seen);
        let work_queue_receiver = work_queue_receiver.clone();
        let work_queue_sender = work_queue_sender.clone();
        let result_queue_sender = result_queue_sender.clone();
        println!("Spawing Thread: {}", (i + 1).to_string());
        let handle = std::thread::spawn(move || loop {
            match work_queue_receiver.recv() {
                Ok(state_to_process) => {
                    let mut seen_set = shared_set_clone.lock().unwrap();
                    let state_to_process_num = board_to_number(state_to_process.clone());
                    if !is_over(&state_to_process) {
                        for next_state in
                            get_all_next_states(state_to_process.board, state_to_process.player)
                        {
                            let next_game = Game {
                                board: next_state,
                                player: switch_player(state_to_process.player),
                            };
                            let next_game_num = board_to_number(next_game.clone());
                            if seen_set.contains(&next_game_num) {
                                continue;
                            }
                            let _ = result_queue_sender.send((state_to_process_num, next_game_num));
                            seen_set.insert(next_game_num);
                            let _ = work_queue_sender.send(next_game);
                        }
                    }
                }
                Err(_) => break,
            }
        });
        handles.push(handle);
    }
    println!("All Threads Spawned");
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("output.txt")
        .expect("Failed to open file");
    println!("Starting File");
    loop {
        match result_queue_receiver.recv() {
            Ok((a, b)) => {
                writeln!(&mut file, "{} {}", a.to_string(), b.to_string()).unwrap();
            }
            Err(_) => {
                println!("stupid!");
                let mut to_break: bool = true;
                for handle in &handles {
                    if !handle.is_finished() {
                        to_break = false;
                        break;
                    }
                }
                if to_break {
                    break;
                }
            }
        }
    }
    println!("Writing Finished");
    for handle in handles {
        handle.join().unwrap();
    }
    drop(work_queue_sender);
    println!("Done");
}
