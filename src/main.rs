use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::io::{BufRead, BufReader, Error, Read, Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex};
extern crate crossbeam;
use crossbeam::channel::unbounded;
use std::fs::{File, OpenOptions};

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

fn game_to_number(g: Game) -> u64 {
    let mut output: u64 = 0;
    for x in 0..3 {
        for y in 0..3 {
            for z in 0..3 {
                output += (g.board.data[x][y][z] as u64) * 3_u64.pow((x + 3 * y + 9 * z) as u32);
            }
        }
    }
    return output;
}

fn board_to_number(g: &Board) -> u64 {
    let mut output: u64 = 0;
    for x in 0..3 {
        for y in 0..3 {
            for z in 0..3 {
                output += (g.data[x][y][z] as u64) * 3_u64.pow((x + 3 * y + 9 * z) as u32);
            }
        }
    }
    return output;
}

fn number_to_board(mut num: u64) -> Game {
    let mut output: Game = Game {
        board: make_new_board(),
        player: 1,
    };
    let mut pieces: i8 = 0;
    for power in 0..27 {
        let value = (num % 3) as i8;
        if value != 0 {
            pieces += 1;
        }
        output.board.data[power % 3][(power / 3) % 3][power / 9] = value;
        num -= num % 3;
        num /= 3;
    }
    output.player = (pieces % 2) + 1;
    return output;
}

fn read_bit_at_position(file_path: &str, position: u64) -> std::io::Result<bool> {
    let mut file = OpenOptions::new().read(true).open(file_path)?;

    file.seek(SeekFrom::Start(position / 8))?;

    let mut buffer = [0u8; 1];
    file.read_exact(&mut buffer)?;

    let byte = buffer[0];
    let bit_position = position % 8;
    let bit_value = byte & (1 << (7 - bit_position));

    Ok(bit_value != 0)
}

fn write_bit_at_position(file_path: &str, position: u64, value: bool) -> std::io::Result<()> {
    let mut file = OpenOptions::new().read(true).write(true).open(file_path)?;

    file.seek(SeekFrom::Start(position / 8))?;

    let mut buffer = [0u8; 1];
    file.read_exact(&mut buffer)?;

    let bit_position = position % 8;
    if value {
        buffer[0] |= 1 << (7 - bit_position);
    } else {
        buffer[0] &= !(1 << (7 - bit_position));
    }

    file.seek(SeekFrom::Start(position / 8))?;
    file.write_all(&buffer)?;

    Ok(())
}

fn get_all_next_numbers(g: Game) -> Vec<u64> {
    get_all_next_states(g.board, g.player)
        .iter()
        .map(|x| board_to_number(x))
        .collect()
}

fn log_base_3(x: f64) -> f64 {
    let result = x.ln() / 3f64.ln();
    result
}

fn get_move_between_board(b1: u64, b2: u64) -> i8 {
    let mut diff = b2 - b1;
    if diff % 2 == 0 {
        diff /= 2;
    }
    log_base_3(diff as f64) as i8
}

fn is_full(g: &Game) -> bool {
    for x in 0..3 {
        for y in 0..3 {
            for z in 0..3 {
                if g.board.data[x][y][z] == 0 {
                    return false;
                }
            }
        }
    }
    true
}

fn minimax_tree() {
    let game_value: Arc<Mutex<HashMap<u64, i8>>> = Arc::new(Mutex::new(HashMap::new()));
    let (work_queue_sender, work_queue_receiver) = unbounded::<u64>();
    let (result_queue_sender, result_queue_receiver) = unbounded::<(u64, i8, i8)>();
    let finished = Arc::new(Mutex::new(false));

    //This thread reads from unqiue.bin in reverse and sends values to worker threads
    let mut file = File::open("C:/Users/evana/Desktop/Connect3/src/unique.bin").unwrap();
    let file_size = file.metadata().unwrap().len();
    let mut position = file_size as i64;
    const CHUNK_SIZE: usize = 8;
    let mut buffer = [0u8; CHUNK_SIZE];
    let work_queue_sender_clone = work_queue_sender.clone();
    let reader_handle = std::thread::spawn(move || loop {
        let chunk_position = if position >= CHUNK_SIZE as i64 {
            position - CHUNK_SIZE as i64
        } else {
            0
        };
        file.seek(SeekFrom::Start(chunk_position as u64)).unwrap();
        let bytes_to_read = if position >= CHUNK_SIZE as i64 {
            CHUNK_SIZE
        } else {
            position as usize
        };
        let bytes_read = file.read(&mut buffer).unwrap();
        if bytes_read == 0 {
            let _ = work_queue_sender_clone.send(0);
            drop(work_queue_sender_clone);
            break;
        }
        let _ = work_queue_sender_clone.send(u64::from_le_bytes(buffer));
        position -= bytes_read as i64;
        if position <= 0 {
            let _ = work_queue_sender_clone.send(0);
            drop(work_queue_sender_clone);
            break;
        }
    });
    println!("Reader Thread Spawned");

    //Minimax worker threads
    let mut worker_handles = Vec::new();
    let n_threads = 14;
    for i in 0..n_threads {
        let work_queue_receiver = work_queue_receiver.clone();
        let work_queue_sender = work_queue_sender.clone();
        let result_queue_sender = result_queue_sender.clone();
        let game_value_clone = game_value.clone();
        let finished_clone = finished.clone();
        println!("Spawing Worker Thread: {}", (i + 1).to_string());
        let handle = std::thread::spawn(move || loop {
            match work_queue_receiver.try_recv() {
                Ok(num_to_process) => {
                    let board: Game = number_to_board(num_to_process);
                    let player = board.player;
                    if is_over(&board) {
                        let winner = switch_player(player);
                        game_value_clone
                            .lock()
                            .unwrap()
                            .insert(num_to_process, winner);
                        let _ = result_queue_sender.send((num_to_process, -1, winner));
                        continue;
                    }
                    if is_full(&board) {
                        game_value_clone.lock().unwrap().insert(num_to_process, 0);
                        let _ = result_queue_sender.send((num_to_process, -1, 0));
                        continue;
                    }
                    let next_game_nums: Vec<u64> = get_all_next_numbers(board);
                    let mut has_one: bool = false;
                    let mut has_two: bool = false;
                    let mut has_tie: bool = false;
                    let mut one_game: Option<u64> = None;
                    let mut two_game: Option<u64> = None;
                    let mut tie_game: Option<u64> = None;
                    let mut chosen_game: Option<u64> = None;
                    let mut chosen_move: i8 = -1;
                    let mut unfinished: bool = false;
                    for num in next_game_nums {
                        let gv = game_value_clone.lock().unwrap();
                        let value = gv.get(&num).unwrap_or(&-1);
                        match value {
                            -1 => {
                                unfinished = true;
                                let _ = work_queue_sender.send(num_to_process);
                                break;
                            }
                            0 => {
                                has_tie = true;
                                tie_game = Some(num);
                            }
                            1 => {
                                has_one = true;
                                one_game = Some(num);
                            }
                            2 => {
                                has_two = true;
                                two_game = Some(num);
                            }
                            _ => panic!("Value Non Normal"),
                        }
                    }
                    if unfinished {
                        continue;
                    }
                    let mut result: i8 = -1;
                    match player {
                        1 => {
                            if has_one {
                                game_value_clone.lock().unwrap().insert(num_to_process, 1);
                                chosen_game = one_game;
                                result = 1;
                            } else if has_tie {
                                game_value_clone.lock().unwrap().insert(num_to_process, 0);
                                chosen_game = tie_game;
                                result = 0;
                            } else if has_two {
                                game_value_clone.lock().unwrap().insert(num_to_process, 2);
                                chosen_game = two_game;
                                result = 2;
                            }
                        }
                        2 => {
                            if has_two {
                                game_value_clone.lock().unwrap().insert(num_to_process, 2);
                                chosen_game = two_game;
                                result = 2;
                            } else if has_tie {
                                game_value_clone.lock().unwrap().insert(num_to_process, 0);
                                chosen_game = tie_game;
                                result = 0;
                            } else if has_one {
                                game_value_clone.lock().unwrap().insert(num_to_process, 1);
                                chosen_game = one_game;
                                result = 1
                            }
                        }
                        _ => panic!("Player Non Normal"),
                    }
                    if chosen_game.is_some() {
                        chosen_move = get_move_between_board(num_to_process, chosen_game.unwrap());
                    } else {
                        println!(
                            "STATE: {}{}",
                            num_to_process,
                            (has_one || has_two || has_tie)
                        );
                        panic!("Chose non-existent state")
                    }
                    let _ = result_queue_sender.send((num_to_process, chosen_move, result));
                    if num_to_process == 0 {
                        *finished_clone.lock().unwrap() = true;
                    }
                }
                Err(_) => match finished_clone.try_lock() {
                    Ok(done) => {
                        if *done {
                            drop(work_queue_sender);
                            break;
                        }
                    }
                    Err(_) => {
                        continue;
                    }
                },
            }
        });
        worker_handles.push(handle);
    }
    println!("All Worker Threads Spawned");

    //This thread writes to output bin and txt files
    let mut output_bin = File::create("output_bin.bin").unwrap();
    let mut output_txt = File::create("output_txt.txt").unwrap();
    let mut written: u32 = 0;
    let writer_handle = std::thread::spawn(move || loop {
        match result_queue_receiver.recv() {
            Ok((a, b, c)) => {
                written += 1;
                if written % 10000000 == 0 {
                    println!("{} / 548,638,747", written);
                }
                let _ = output_bin.write_all(&a.to_le_bytes());
                let _ = output_bin.write_all(&b.to_le_bytes());
                let _ = output_bin.write_all(&c.to_le_bytes());
                let content = a.to_string() + &" " + &b.to_string() + &" " + &c.to_string() + &"\n";
                let _ = output_txt.write_all(content.as_bytes());
            }
            Err(_) => {
                break;
            }
        }
    });
    println!("Writer Thread Spawned");
    reader_handle.join().unwrap();
    println!("Reading Done");
    drop(work_queue_sender);
    for handle in worker_handles {
        handle.join().unwrap();
    }
    println!("Working Done");
    writer_handle.join().unwrap();
    drop(result_queue_sender);
    println!("Writing Done");
}

fn get_best_move(state_num: u64) -> Option<(i8, i8)> {
    let mut file = OpenOptions::new()
        .read(true)
        .open("C:/Users/evana/Desktop/Connect3/output_sorted_bin.bin")
        .unwrap();
    let mut low: i64 = -1;
    let mut high: i64 = 548638748;
    while high > low + 1 {
        let mid: i64 = (low + high) / 2;
        file.seek(SeekFrom::Start((mid as u64) * 10)).unwrap();
        let mut buffer = [0u8; 8];
        file.read_exact(&mut buffer).unwrap();
        let mut int64_bytes: [u8; 8] = Default::default();
        int64_bytes.copy_from_slice(&buffer[0..8]);
        let found_state_num = u64::from_le_bytes(int64_bytes);
        if found_state_num == state_num {
            file.seek(SeekFrom::Start((mid as u64) * 10 + 8)).unwrap();
            let mut buffer = [0u8; 2];
            file.read_exact(&mut buffer).unwrap();
            let mut first_byte: i8 = buffer[0] as i8;
            let mut second_byte: i8 = buffer[1] as i8;
            return Some((first_byte, second_byte));
        } else if found_state_num > state_num {
            high = mid;
        } else {
            low = mid;
        }
    }
    return None;
}

fn sort_output() {
    let mut file = File::open("C:/Users/evana/Desktop/Connect3/output_bin.bin").unwrap();
    let mut output_sorted = File::create("output_sorted_bin.bin").unwrap();
    let mut buffer = [0; 10];
    let mut output_vec: Vec<(u64, i8, i8)> = Vec::new();
    while let Ok(_) = file.read_exact(&mut buffer) {
        let mut int64_bytes: [u8; 8] = Default::default();
        int64_bytes.copy_from_slice(&buffer[0..8]);
        let state_num = u64::from_le_bytes(int64_bytes);
        let next_move: i8 = buffer[8] as i8;
        let game_value: i8 = buffer[9] as i8;
        output_vec.push((state_num, next_move, game_value));
    }
    println!("Data Loaded");
    output_vec.sort_unstable_by_key(|&(state_num, _, _)| state_num);
    println!("Sorted");
    for (a, b, c) in output_vec {
        let _ = output_sorted.write_all(&a.to_le_bytes());
        let _ = output_sorted.write_all(&b.to_le_bytes());
        let _ = output_sorted.write_all(&c.to_le_bytes());
    }
    println!("Sorted Data Written");
}

fn stored_move_to_human_move(stored_move: i8) -> i8 {
    if stored_move == -1 {
        return -1;
    }
    stored_move / 3
}

fn game_to_str(g: Game) -> String {
    let mut output: String = "".to_string();
    for z in 0..3 {
        for x in 0..3 {
            let content = g.board.data[x][0][z].to_string()
                + &" "
                + &g.board.data[x][1][z].to_string()
                + &" "
                + &g.board.data[x][2][z].to_string()
                + &"\n";
            output.push_str(&content);
        }
        output.push_str(&"\n");
    }
    return output;
}

fn main() {
    loop {
        let mut input = String::new();
        println!("Enter a State Number:");
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let number: Result<i64, _> = input.trim().parse();
        match number {
            Ok(parsed_number) => {
                let (output, winner) = get_best_move(parsed_number as u64).unwrap();
                let output = stored_move_to_human_move(output);
                let mut g: Game = number_to_board(parsed_number as u64);
                if output != -1 {
                    let _ = place_new_piece(
                        &mut g.board,
                        (output / 3) as usize,
                        (output % 3) as usize,
                        g.player,
                    );
                }
                let next_number: u64 = board_to_number(&g.board);
                println!(
                    "Down: {}, Right: {}, Next State: {}, Winner: {}",
                    (output / 3).to_string(),
                    (output % 3).to_string(),
                    next_number.to_string(),
                    winner.to_string()
                );
                println!("{}", game_to_str(g));
            }
            Err(_) => {
                println!("Failed to parse an integer, quitting");
                break;
            }
        }
    }
}

fn generate() {
    let seen: Arc<Mutex<HashSet<u64>>> = Arc::new(Mutex::new(HashSet::new()));
    let mut handles = Vec::new();
    let (work_queue_sender, work_queue_receiver) = unbounded::<Game>();
    let (result_queue_sender, result_queue_receiver) = unbounded::<u64>();
    let _ = result_queue_sender.send(0);
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
                    if !is_over(&state_to_process) {
                        for next_state in
                            get_all_next_states(state_to_process.board, state_to_process.player)
                        {
                            let next_game = Game {
                                board: next_state,
                                player: switch_player(state_to_process.player),
                            };
                            let next_game_num = board_to_number(&next_game.board);
                            if seen_set.contains(&next_game_num) {
                                continue;
                            }
                            let _ = result_queue_sender.send(next_game_num);
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
    println!("All Worker Threads Spawned");
    let mut file = File::create("unique.bin").unwrap();
    println!("Starting File Writing");
    loop {
        match result_queue_receiver.recv() {
            Ok(a) => {
                let _ = file.write_all(&a.to_le_bytes());
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
    for handle in handles {
        handle.join().unwrap();
    }
    drop(work_queue_sender);
    println!("Done");
}
