use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
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

fn write_unique() {
    let mut file = File::open("data.bin").unwrap();
    let mut write_file = File::create("unique.bin").unwrap();
    let mut buffer = [0; 8];
    let mut seen: HashSet<u64> = HashSet::new();
    while let Ok(_) = file.read_exact(&mut buffer) {
        let num1 = u64::from_le_bytes(buffer);
        file.read_exact(&mut buffer).unwrap();
        let num2 = u64::from_le_bytes(buffer);
        if !seen.contains(&num2) {
            seen.insert(num2);
            let  _ = write_file.write_all(&num2.to_le_bytes());
        }
    }
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
    let result = (x.ln() / 3f64.ln());
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
    let mut handles = Vec::new();
    let n_threads = 16;
    for i in 0..n_threads {
        let work_queue_receiver = work_queue_receiver.clone();
        let work_queue_sender = work_queue_sender.clone();
        let result_queue_sender = result_queue_sender.clone();
        let game_value_clone = game_value.clone();
        println!("Spawing Thread: {}", (i + 1).to_string());
        let handle = std::thread::spawn(move || loop {
            match work_queue_receiver.recv() {
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
                        println!("STATE: {}{}", num_to_process, (has_one || has_two || has_tie));
                        panic!("Chose non-existent state")
                    }
                    let _ = result_queue_sender.send((num_to_process, chosen_move, result));
                }
                Err(_) => break,
            }
        });
        handles.push(handle);
    }
    println!("All Threads Spawned");
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
            break; // Reached the beginning of the file
        }

        let _ = work_queue_sender_clone.send(u64::from_le_bytes(buffer));

        position -= bytes_read as i64;
        if position <= 0 {
            break; // Ensure we don't read beyond the beginning of the file
        }
    });
    let mut output = File::create("output.bin").unwrap();
    let mut n: u32 = 0;
    loop {
        match result_queue_receiver.recv() {
            Ok((a, b, c)) => {
                n += 1;
                if (n % 5000000 == 0) {
                    println!("{}", n);
                }
                let _ = output.write_all(&a.to_le_bytes());
                let _ = output.write_all(&b.to_le_bytes());
                let _ = output.write_all(&c.to_le_bytes());
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
    reader_handle.join().unwrap();
    work_queue_sender.send(0);
    for handle in handles {
        handle.join().unwrap();
    }
    drop(work_queue_sender);
    println!("Done");
}

fn read_output_to_text() {
    let mut file = File::open("C:/Users/evana/Desktop/Connect3/output.bin").unwrap();
    let mut output = File::create("output.txt").unwrap();
    let mut buffer = [0; 10];
    let mut seen: HashSet<u64> = HashSet::new();
    while let Ok(_) = file.read_exact(&mut buffer) {
        let mut int64_bytes: [u8; 8] = Default::default();
        int64_bytes.copy_from_slice(&buffer[0..8]);
        let state_num = u64::from_le_bytes(int64_bytes);
        let next_move : i8 = buffer[8] as i8;
        let game_value : i8 = buffer[9] as i8;
        let content = state_num.to_string() + &" " + &next_move.to_string() + &" " + &game_value.to_string() + &"\n";
        output.write_all(content.as_bytes()).unwrap();
    }
}

fn main() {
    read_output_to_text();
}

fn _process() {
    let file = File::open("C:\\Users\\evana\\Desktop\\Connect3\\input.txt").unwrap();
    let reader = BufReader::new(file);
    let mut output = File::create("data.bin").unwrap();
    let mut n: u32 = 0;
    for line in reader.lines() {
        if let Ok(line) = line {
            n += 1;
            if (n % 5000000 == 0) {
                println!("{}", n);
            }
            let substrings: Vec<&str> = line.split_whitespace().collect();
            let _ = output.write_all(&substrings[0].parse::<u64>().unwrap().to_le_bytes());
            let _ = output.write_all(&substrings[1].parse::<u64>().unwrap().to_le_bytes());
        }
    }
}

fn generate() {
    let seen: Arc<Mutex<HashSet<u64>>> = Arc::new(Mutex::new(HashSet::new()));
    let mut handles = Vec::new();
    let (work_queue_sender, work_queue_receiver) = unbounded::<Game>();
    let (result_queue_sender, result_queue_receiver) = unbounded::<(u64, u64)>();
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
                    let state_to_process_num = board_to_number(&state_to_process.board);
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
    let mut file = File::create("data.bin").unwrap();
    println!("Starting File");
    loop {
        match result_queue_receiver.recv() {
            Ok((a, b)) => {
                let _ = file.write_all(&a.to_le_bytes());
                let _ = file.write_all(&b.to_le_bytes());
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
