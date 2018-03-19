extern crate nix;
use nix::sys::termios::{Termios, tcgetattr, tcsetattr, LocalFlags, SetArg};
use nix::libc::*;
extern crate rand;
use rand::Rng;
use std::io::{stdout, Write};

fn set_non_canon() -> Termios {
    let old_info = tcgetattr(0).unwrap();
    let mut new_info = old_info.clone();
    new_info.local_flags.remove(LocalFlags::from_bits(ICANON | ECHO).unwrap());
    new_info.control_chars[VMIN] = 1;
    new_info.control_chars[VTIME] = 0;
    tcsetattr(0, SetArg::TCSANOW, &new_info).unwrap();
    old_info
}

fn set_canonical (old: &Termios) {
    tcsetattr(0, SetArg::TCSANOW, old).unwrap();
}

fn print_border() {
    let horiz_line = "--------------------------------------------".to_string();
    // Print horizontal lines
    for r in 1..4 {
        print!("[{};1H{}", r * 6, horiz_line);
    }

    // Print vertically
    for c in 1..5 {
        for r in 1..24 {
            print!("[{};{}H|", r, 11 * c);
        }
    }

    // Print dot
    for c in 1..5 {
        for r in 1..4 {
            print!("[{};{}H+", r * 6, c * 11);
        }
    }
}

fn flush_stdout() {
    stdout().flush().unwrap();
}

fn print_tiles(tiles: &[u16; 16]) {
    for (tile_no, num) in tiles.iter().enumerate() {
        let color = match *num {
            2 => "[48;2;134;222;132m",
            4 => "[48;2;103;204;252m",
            8 => "[48;2;153;51;255m",
            16 => "[48;2;255;154;154m",
            32 => "[48;2;255;227;153m",
            64 => "[48;2;163;205;73m",
            128 => "[48;2;13;169;175m",
            256 => "[48;2;108;56;153m",
            512 => "[48;2;238;197;221m",
            1024 => "[48;2;148;24;24m",
            2048 => "[48;2;0;0;0m",
            4096 => "[48;2;200;200;200m",
            _ => "",
        };

        let pos_y = (tile_no / 4) * 6 + 1;
        let pos_x = (tile_no % 4) * 11 + 1;
        let num_str = if *num == 0 { "".to_string() } else { format!("{}", num) };
        print!("{}", color);
        for i in 0..5 {
            if i == 2 {
                print!("[{};{}H{: ^10}",   pos_y + 2, pos_x, num_str);
            } else {
                print!("[{};{}H          ", pos_y + i, pos_x);
            }
        }
        print!("[0m");
    }
}

fn getch() -> char {
    unsafe {
        getchar() as u8 as char
    }
}

fn get_key() -> char {
    let mut signal: char;
    let key: char;
    let escape: bool;
    signal = getch() as char;
    escape = signal == '' && getch() == '[';
    if escape {
        signal = getch() as char;
    }

    if escape {
        key = match signal {
            'A' => 'U',
            'B' => 'D',
            'C' => 'R',
            'D' => 'L',
            _   => 'x',
        }
    } else {
        key = match signal {
            'w' | 'W'             => 'U',
            's' | 'S'             => 'D',
            'd' | 'D'             => 'R',
            'a' | 'A'             => 'L',
            'r' | 'R'             => 'r',
            'q' | 'Q'             => 'q',
            'i' | 'I' | 'h' | 'H' => 'h',
            _                     => 'x',
        }
    }
    key
}

fn move_tile(tiles: &mut [u16; 16], dir: char) -> u32 {
    let mut delta = 0u32;

    match dir {
        'U' => {
            for j in 0..6 {
                if j != 4 {
                    for i in 0..12 {
                        if tiles[i] == 0 {
                            tiles[i] = tiles[i + 4];
                            tiles[i + 4] = 0;
                        }
                    }
                } else {
                    for i in 0..12 {
                        if tiles[i] == tiles[i + 4] {
                            tiles[i] = 2 * tiles[i];
                            tiles[i + 4] = 0;
                            delta += tiles[i] as u32;
                        }
                    }
                }
            }
        },
        'D' => {
            for j in 0..6 {
                if j != 4 {
                    for i in (0..12).rev() {
                        if tiles[i + 4] == 0 {
                            tiles[i + 4] = tiles[i];
                            tiles[i] = 0;
                        }
                    }
                } else {
                    for i in (0..12).rev() {
                        if tiles[i] == tiles[i + 4] {
                            tiles[i + 4] = 2 * tiles[i + 4];
                            tiles[i] = 0;
                            delta += tiles[i + 4] as u32;
                        }
                    }
                }
            }
        },
        'R' => {
            for j in 0..6 {
                if j != 4 {
                    for i in (0..15).rev() {
                        match i {
                            3 | 7 | 11 => continue,
                            _          => (),
                        }
                        if tiles[i + 1] == 0 {
                            tiles[i + 1] = tiles[i];
                            tiles[i] = 0;
                        }
                    }
                } else {
                    for i in (0..15).rev() {
                        match i {
                            3 | 7 | 11 => continue,
                            _          => (),
                        }
                        if tiles[i] == tiles[i + 1] {
                            tiles[i + 1] = 2 * tiles[i + 1];
                            tiles[i] = 0;
                            delta += tiles[i + 1] as u32;
                        }
                    }
                }
            }
        },
        'L' => {
            for j in 0..6 {
                if j != 4 {
                    for i in 0..15 {
                        match i {
                            3 | 7 | 11 => continue,
                            _          => (),
                        }
                        if tiles[i] == 0 {
                            tiles[i] = tiles[i + 1];
                            tiles[i + 1] = 0;
                        }
                    }
                } else {
                    for i in 0..15 {
                        match i {
                            3 | 7 | 11 => continue,
                            _          => (),
                        }
                        if tiles[i] == tiles[i + 1] {
                            tiles[i] = 2 * tiles[i];
                            tiles[i + 1] = 0;
                            delta += tiles[i] as u32;
                        }
                    }
                }
            }
        },
        _ => (),
    }
    delta
}

fn place_tile(tiles: &mut [u16; 16]) {
    let mut rng = rand::thread_rng();
    let mut empty_tiles = [0u8; 16];
    let mut tmp = 0u8;

    for i in 0..16 {
        if tiles[i] == 0 {
            empty_tiles[tmp as usize] = i as u8;
            tmp += 1;
        }
    }

    if tmp > 0 {
        tiles[empty_tiles[rng.gen_range(0, tmp) as usize] as usize] = 
                if rng.gen_range(0, 10) % 10 == 0 { 4 } else { 2 };
    }
}

fn print_status(score: u32) {
    print!("[24;1H");
    print!("[1;7m");
    print!("{: >80}\r{: >70}\r{: >60}\r{}", "          ", score, "YOUR SCORE: ", "-- THE 2048 GAME --");
    print!("[24;80H[0m");
}

fn print_info(status: u8) {
    match status {
        0 => {      // Normal
            print! ("[3;45H{}", "    Use your arrow key to play      ");
            print! ("[4;45H{}", "                                    ");
            print! ("[5;45H{}", "               _____                ");
            print! ("[6;45H{}", "              |     |               ");
            print! ("[7;45H{}", "              |  ^  |               ");
            print! ("[8;45H{}", "              |  |  |               ");
            print! ("[9;45H{}", "        -------------------         ");
            print!("[10;45H{}", "        |     |     |     |         ");
            print!("[11;45H{}", "        |  <- |  |  | ->  |         ");
            print!("[12;45H{}", "        |     |  v  |     |         ");
            print!("[13;45H{}", "        -------------------         ");
            print!("[14;45H{}", "       Or W, S, A, D instead        ");
        },
        1 => {      // Win
            for i in 0..9 {
                print! ("[{};45H{}", 3 + i, "                                    ");
            }
            print!("[34m");
            print!("[12;45H{}", "              HURRAY!!!             ");
            print!("[1;35m");
            print!("[13;45H{}", "         You reached 2048!!         ");
            print!("[0m");
            print!("[14;45H{}", "          Move to continue          ");
        },
        2 => {      // Lost
            for i in 0..10 {
                print! ("[{};45H{}", 3 + i, "                                    ");
            }
            print!("[1;31m");
            print!("[13;45H{}", "               OH NO!!              ");
            print!("[0m");
            print!("[14;45H{}", "             You lose!!!            ");
        },
        _ => (),
    }
    print!("[20;45H{}", "        Press 'r' to retry");
    print!("[21;45H{}", "              'q' to quit");
}

fn check_end(tiles: &[u16; 16]) -> bool {
    let mut copy = [0u16; 16];
    copy.copy_from_slice(tiles);

    if copy.iter().any(|e| *e == 0) {
        return false;
    }

    move_tile(&mut copy, 'U');
    move_tile(&mut copy, 'D');
    move_tile(&mut copy, 'R');
    move_tile(&mut copy, 'L');

    copy == *tiles
}

fn main() {
    print!("[s");             // Store cursor position
    print!("[?1049h");        // Store window in buffer
    print!("[2J");            // Clear screen
    print!("[?25l");
    print_border();
    let saved_state = set_non_canon();

    'game_loop: loop {
        let mut reached = false;
        let mut score = 0u32;
        let mut tiles = [0u16; 16];

        place_tile(&mut tiles);
        place_tile(&mut tiles);
        print_info(0);
        print_status(score);
        print_tiles(&tiles);
        flush_stdout();

        'main_loop: loop {
            if check_end(&tiles) {
                break 'main_loop;
            }

            //check 2048
            if !reached {
                if tiles.iter().any(|e| *e == 2048) {
                    print_info(1);
                    reached = true;
                }
            } else {
                print_info(0);
            }

            // Get key
            let key = get_key();
            match key {
                'U' | 'D' | 'R' | 'L' => score += move_tile(&mut tiles, key),
                'q' => break 'game_loop,
                'r' => continue 'game_loop,
                _ => (),
            }

            place_tile(&mut tiles);
            print_tiles(&tiles);
            print_status(score);
            flush_stdout();
        }

        print_info(2);
        flush_stdout();
        loop {
            match get_key() {
                'q' => break 'game_loop,
                'r' => continue 'game_loop,
                _ => (),
            }
        }
    }

    // Exit
    set_canonical(&saved_state);// Reset terminal
    print!("[?25h");
    print!("[u\n");          // Restore cursor position
    print!("[?1049l");       // Restore the terminal window
}
