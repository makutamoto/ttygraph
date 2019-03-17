extern crate rand;
use rand::prelude::*;
#[macro_use] extern crate log;
extern crate simplelog;
use simplelog::{ Config, LevelFilter, WriteLogger };
extern crate ncurses;
extern crate regex;
use ncurses::*;
use std::i16;
use std::fs::File;

mod formula;

const DEFAULT_GRAPH_COLOR: i16 = 2;
const X_COLOR: i16 = 3;
const Y_COLOR: i16 = 4;
const ERROR_MESSAGE: i16 = 5;
const GRAPH_COLOR_START: i16 = 6;

#[derive(Debug)]
enum Mode {
    Edit,
    Normal,
}

fn main() {
    let mut nof_graphs_ever = 0;
    let mut mode = Mode::Normal;
    let mut size;
    let mut cursor = 0;
    let mut input = (0usize, String::new());
    let mut error = String::new();
    let mut center = [0.0; 2];
    let mut scale = 0;
    let mut unit = 1.0;
    let mut formulae = Vec::<formula::Formula>::new();
    let mut history = 0i32;
    let mut graph_buffer;
    if cfg!(debug_assertions) { WriteLogger::init(LevelFilter::Trace, Config::default(), File::create("log.txt").unwrap()).unwrap(); };
    initscr();
    if has_colors() {
        start_color();
        use_default_colors();
        init_pair(DEFAULT_GRAPH_COLOR, COLOR_BLUE, -1);
        init_pair(X_COLOR, COLOR_RED, -1);
        init_pair(Y_COLOR, COLOR_GREEN, -1);
        init_pair(ERROR_MESSAGE, COLOR_RED, -1);
    }
    raw();
    noecho();
    {
        let mut width = 0;
        let mut height = 0;
        getmaxyx(stdscr(), &mut height, &mut width);
        size = [width, height];
    }
    graph_buffer = refresh_buffer(size, unit, center, &formulae);
    loop {
        draw(&mode, size, scale, center, cursor, &input.1, &error, &formulae, &graph_buffer);
        let key = getch();
        if key == KEY_RESIZE {
            {
                let mut width = 0;
                let mut height = 0;
                getmaxyx(stdscr(), &mut height, &mut width);
                size = [width, height];
            }
            info!("display size: ({}, {})", size[0], size[1]);
            graph_buffer = refresh_buffer(size, unit, center, &formulae);
        } else {
            if key <= 0x1F {
                let key = (key + 0x40) as u8 as char;
                match key {
                    'A' => {
                        match mode {
                            Mode::Normal => {
                                input.0 = formulae.len();
                                cursor = 0;
                                mode = Mode::Edit;
                            },
                            _ => (),
                        }
                    },
                    'C' => {
                        match mode {
                            Mode::Normal => {
                                center = [0.0, 0.0];
                                graph_buffer = refresh_buffer(size, unit, center, &formulae);
                            },
                            Mode::Edit => mode = Mode::Normal,
                        }
                        input.1.clear();
                    },
                    'D' => {
                        match mode {
                            Mode::Normal => {
                                match graph_buffer[size[1] as usize / 2][get_physical_center_x(size) as usize / 2] {
                                    Some(id) => {
                                        formulae.remove(id);
                                        graph_buffer = refresh_buffer(size, unit, center, &formulae);
                                    },
                                    _ => (),
                                }
                            },
                            _ => (),
                        }
                    },
                    'E' => {
                        match mode{
                            Mode::Normal => {
                                match graph_buffer[size[1] as usize / 2][get_physical_center_x(size) as usize / 2] {
                                    Some(id) => {
                                        input.0 = id;
                                        input.1 = formulae[id].get_raw().to_string();
                                        cursor = input.1.len();
                                        history = 0;
                                        mode = Mode::Edit;
                                    },
                                    _ => (),
                                }
                            },
                            _ => (),
                        }
                    },
                    'I' => if scale > -9 {
                        match mode {
                            Mode::Normal => {
                                scale -= 1;
                                unit = 10.0_f64.powi(scale);
                                center[0] -= center[0] % unit;
                                center[1] -= center[1] % unit;
                                graph_buffer = refresh_buffer(size, unit, center, &formulae);
                            },
                            _ => (),
                        }
                    },
                    'J' => {
                        match mode {
                            Mode::Normal => { beep(); },
                            Mode::Edit => {
                                let color_pair = {
                                    if input.0 == formulae.len() {
                                        let pair = nof_graphs_ever + GRAPH_COLOR_START;
                                        let color = if can_change_color() {
                                            let color = nof_graphs_ever + 12;
                                            init_color(color, rand::thread_rng().gen_range(0, 1001), rand::thread_rng().gen_range(0, 1001), 1000);
                                            color
                                        } else {
                                            DEFAULT_GRAPH_COLOR
                                        };
                                        init_pair(pair, color, -1);
                                        pair
                                    } else {
                                        let pair = formulae[input.0].get_color_pair();
                                        formulae.remove(input.0);
                                        pair
                                    }
                                };
                                match formula::Formula::new(&input.1.to_string(), color_pair) {
                                    Ok(formula) => {
                                        formulae.insert(input.0, formula);
                                        mode = Mode::Normal;
                                        graph_buffer = refresh_buffer(size, unit, center, &formulae);
                                        input.1.clear();
                                        error.clear();
                                    },
                                    Err(e) => error = String::from(e),
                                }
                                nof_graphs_ever += 1;
                            },
                        }
                    },
                    'O' => if scale < 9 {
                        match mode {
                            Mode::Normal => {
                                scale += 1;
                                unit = 10.0_f64.powi(scale);
                                center[0] -= center[0] % unit;
                                center[1] -= center[1] % unit;
                                graph_buffer = refresh_buffer(size, unit, center, &formulae);
                            },
                            _ => (),
                        }
                    },
                    'X' => {
                        break;
                    },
                    '[' => {
                        if getch() == 0x5B {
                            let move_unit = 10.0_f64.powi(scale);
                            match getch() as u8 as char {
                                'A' => {
                                    match mode {
                                        Mode::Normal => {
                                            let mut line = Vec::new();
                                            center[1] += move_unit;
                                            for x in 0..graph_buffer[0].len() as i32 {
                                                line.push(eval_for_dot(&formulae, x, 0, size, unit, center));
                                            }
                                            graph_buffer.insert(0, line);
                                            graph_buffer.pop();
                                        },
                                        Mode::Edit => {
                                            if input.0 as i32 - history > 0 {
                                                history += 1;
                                                input.1 = formulae[(input.0 as i32 - history) as usize].get_raw().to_string();
                                                cursor = input.1.len();
                                            }
                                        },
                                    }
                                },
                                'B' => {
                                    match mode {
                                        Mode::Normal => {
                                            let mut line = Vec::new();
                                            center[1] -= move_unit;
                                            for x in 0..graph_buffer[0].len() as i32 {
                                                line.push(eval_for_dot(&formulae, x, graph_buffer.len() as i32 - 1, size, unit, center));
                                            }
                                            graph_buffer.remove(0);
                                            graph_buffer.push(line);
                                        },
                                        Mode::Edit => {
                                            if input.0 as i32 - history < formulae.len() as i32 {
                                                history -= 1;
                                                let index = (input.0 as i32 - history) as usize;
                                                if index == formulae.len() {
                                                    input.1.clear();
                                                    cursor = 0;
                                                } else {
                                                    input.1 = formulae[index].get_raw().to_string();
                                                    cursor = input.1.len();
                                                }
                                            }
                                        },
                                    }
                                },
                                'C' => {
                                    match mode {
                                        Mode::Normal => {
                                            let width = graph_buffer[0].len() as i32;
                                            center[0] += move_unit;
                                            for y in 0..graph_buffer.len() {
                                                graph_buffer[y].push(eval_for_dot(&formulae, width - 1, y as i32, size, unit, center));
                                                graph_buffer[y].remove(0);
                                            }
                                        },
                                        Mode::Edit => {
                                            if cursor < input.1.len() {
                                                cursor += 1;
                                            } else {
                                                beep();
                                            }
                                        },
                                    }
                                },
                                'D' => {
                                    match mode {
                                        Mode::Normal => {
                                            center[0] -= move_unit;
                                            for y in 0..graph_buffer.len() {
                                                graph_buffer[y].pop();
                                                graph_buffer[y].insert(0, eval_for_dot(&formulae, 0, y as i32, size, unit, center));
                                            }
                                        },
                                        Mode::Edit => {
                                            if cursor > 0 {
                                                cursor -= 1
                                            } else {
                                                beep();
                                            }
                                        },
                                    }
                                },
                                _ => (),
                            }
                        }
                    }
                    _ => { beep(); },
                }
            } else {
                match mode {
                    Mode::Normal => { beep(); },
                    Mode::Edit => {
                        if key == 0x7F {
                            if cursor > 0 {
                                cursor -= 1;
                                input.1.remove(cursor as usize);
                            } else {
                                beep();
                            }
                        } else {
                            input.1.insert(cursor as usize, key as u8 as char);
                            cursor += 1;
                        }
                    }
                }
            }
        }
    }
    endwin();
}

fn draw(mode: &Mode, size: [i32; 2], scale: i32, center: [f64; 2], cursor: usize, input: &String, error: &str, formulae: &Vec<formula::Formula>, graph_buffer: &Vec<Vec<Option<usize>>>) {
    info!("DRAW_CALL");
    let scale_abs = 10_i32.pow(scale.abs() as u32);
    let nof_dec_places = if scale < 0 { scale.abs() as usize } else { 0 };
    let label = format!("scale: {}, x: {:.*}, y: {:.*}", if scale > 0 { format!("1/{}", scale_abs) } else { scale_abs.to_string() }, nof_dec_places, center[0], nof_dec_places, center[1]);
    let unit = 10.0_f64.powi(scale);
    let physical_center = (get_physical_center_x(size), size[1] / 2);
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    for x in 0..size[0] {
        for y in 0..size[1] {
            if x % 2 != 0 {
                mvprintw(y, x, " ");
                continue;
            }
            let cord_x = (((x - size[0] / 2) as f64 / 2.0).ceil()  + (center[0] / unit).round()) as i32;
            let cord_y = size[1] / 2 - y + (center[1] / unit).round() as i32;
            let center_x = cord_x == 0;
            let center_y = cord_y == 0;
            attr_on(A_BOLD());
            if center_x && center_y {
                mvaddch(y, x, ACS_PLUS());
            } else if center_y {
                if cord_x % 2 == 0{
                    mvaddch(y, x, ACS_HLINE());
                } else {
                    mvprintw(y, x, " ");
                }
            } else if center_x {
                if cord_y % 2 == 0 {
                    mvaddch(y, x, ACS_VLINE());
                } else {
                    mvprintw(y, x, " ");
                }
            } else if cord_x % 5 == 0 && cord_y % 5 == 0 {
                mvaddch(y, x, ACS_BULLET());
            } else {
                mvprintw(y, x, " ");
            }
            attr_off(A_BOLD());
            if let Some(id) = graph_buffer[y as usize][(x / 2) as usize] {
                attron(COLOR_PAIR(formulae[id].get_color_pair()));
                mvprintw(y, x, "*");
                attroff(COLOR_PAIR(formulae[id].get_color_pair()));
            }
            if center_x && y == 0 {
                attron(COLOR_PAIR(Y_COLOR) | A_BOLD());
                mvprintw(y, x, "Y");
                attroff(COLOR_PAIR(Y_COLOR) | A_BOLD());
            } else if center_y && (x == size[0] - 1 || x == size[0] - 2) {
                attron(COLOR_PAIR(X_COLOR) | A_BOLD());
                mvprintw(y, x, "X");
                attroff(COLOR_PAIR(X_COLOR) | A_BOLD());
            }
        }
    }
    mvprintw(physical_center.1, physical_center.0, "#");
    mvprintw(size[1] - 1, size[0] - label.len() as i32, &label);
    if let Some(id) = graph_buffer[physical_center.1 as usize][physical_center.0 as usize / 2] {
        let formula = formulae[id].get_raw();
        mvprintw(physical_center.1, physical_center.0 + 2, "(");
        print_formula(physical_center.0 + 3, physical_center.1, formula);
        mvprintw(physical_center.1, physical_center.0 + 3 + formula.len() as i32, ")");
    }
    match mode {
        Mode::Normal => { mvprintw(size[1] - 1, 0, "^X: quit ^A: add ^E: edit ^D: delete ^C: center ^I/O: zoom in/out"); },
        Mode::Edit => {
            let label = "^X: quit ^C: cancel [formula]: ";
            attron(COLOR_PAIR(ERROR_MESSAGE));
            mvprintw(size[1] - 2, 0, error);
            attroff(COLOR_PAIR(ERROR_MESSAGE));
            mvprintw(size[1] - 1, 0, label);
            print_formula(label.len() as i32, size[1] - 1, &input);
            mv(size[1] - 1, (label.len() + cursor) as i32);
            curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
        },
    }
    refresh();
}

fn get_physical_center_x(size: [i32; 2]) -> i32 {
    let half = size[0] / 2;
    if half % 2 == 0 {
        half
    } else {
        half - 1
    }
}

fn refresh_buffer(size: [i32; 2], unit: f64, center: [f64; 2], formulae: &Vec<formula::Formula>) -> Vec<Vec<Option<usize>>> {
    let mut buffer = Vec::new();
    for y in 0..size[1] as i32 {
        let mut line = Vec::new();
        for x in 0..(size[0] as f64 / 2.0).ceil() as i32 {
            line.push(eval_for_dot(&formulae, x, y, size, unit, center));
        }
        buffer.push(line);
    }
    buffer
}

fn print_formula(x: i32, y: i32, formula: &str) {
    let formula = formula.to_string();
    for (i, character) in formula.chars().enumerate() {
        let character = match character {
            '%' => String::from("%%"),
            _ => character.to_string(),
        };
        mvprintw(y, x + i as i32, &character);
    }
}

fn eval_for_dot(formulae: &Vec<formula::Formula>, x: i32, y: i32, size: [i32; 2], unit: f64, center: [f64; 2]) -> Option<usize> {
    let cords = [(x - get_physical_center_x(size) / 2) as f64 * unit + center[0], (size[1] / 2 - y) as f64 * unit + center[1]];
    for id in (0..formulae.len()).rev() {
        let left = formulae[id].left.calc(cords[0], cords[1]);
        let right = formulae[id].right.calc(cords[0], cords[1]);
        if left.is_ok() && right.is_ok() && (left.unwrap() / unit).round() == (right.unwrap() / unit).round() {
            return Some(id);
        }
    }
    None
}
