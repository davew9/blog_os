use lazy_static::lazy_static;
use spin::Mutex;
use alloc::prelude::v1::{String, ToOwned};
use crate::vga_buffer::print_bytes;
use crate::filesystem;


lazy_static! {
    static ref CLI: Mutex<Cli> = Mutex::new(Cli::new());
}


struct Cli {
    working_dir: String,
    command: [u8;240],
    cursor_pos: usize,
}

impl Cli {
    pub const fn new() -> Cli {
        Cli {
            command: [0; 240],
            working_dir: String::new(),
            cursor_pos: 0
        }
    }
}

pub fn add_char(char: &str) {
    if char == "\n" {
        parse_command();
        return
    }
    let mut cli = CLI.lock();
    let cursor = cli.cursor_pos;
    cli.command[cursor] = (*char).as_bytes()[0];
    cli.cursor_pos += 1;
    if cli.cursor_pos == 240 {
        cli.command = [0;240];
        cli.cursor_pos = 0;
        println!("Buffer overflowed");
        print!("{}>", cli.working_dir);
    }
}

pub fn parse_command() {
    let mut cli = CLI.lock();
    let wd = cli.working_dir.clone();
    let cursor = cli.cursor_pos;
    let command = String::from_utf8_lossy(&cli.command[0..cursor]);

    let mut chunks = command.split(" ");
    let mut operation ="";
    let mut argument = "";
    let mut argument2 = String::new();
    let path;
    let mut pos = 0;

    while let Some(chunk) = chunks.next() {
        if pos == 0 {
            operation = chunk;
        }
        else if pos == 1 {
            argument = chunk;
        }

        else  {
            argument2 = argument2.to_owned() + " " + chunk;
        }
        pos +=1;
    }


    if argument.starts_with("/") {
        path = argument.to_owned();
    }

    else {
        path = wd.trim_end_matches("/").to_owned() + "/" + argument;
    }

    if operation == "cd" { if filesystem::validate_path(&path)  {
        cli.working_dir = path;
    }
    }

    else if operation == "mkdir" {mkdir(&path)}
    else if operation == "show" {show(&path)}
    else if operation == "rm" {rm(&path)}
    else if operation == "mkfile" {mkfile(&path)}
    else if operation == "edit" {edit(&path, &argument2)}
    else {println!("Unknown command")}

    cli.command = [0;240];
    cli.cursor_pos = 0;
    print!("{}>", cli.working_dir);
}


fn show(path: &str ) {
    let fd = filesystem::open(path);
    match fd {
        None => return,
        Some(fd) => {
            let content = filesystem::read(fd, 0).unwrap();
            print_bytes(&content);
            filesystem::close(fd);
        }
    }
}

fn mkdir(path: &str) {
    filesystem::create_dir(path);
}

fn rm(path: &str) {
    filesystem::delete(path);
}

fn mkfile(path: &str) {
    filesystem::create(path);
}

fn edit(path: &str, content: &str ) {
    let fd = filesystem::open(path);
    match fd {
        None => return,
        Some(fd) => {
            filesystem::write(fd, 0, content);
            filesystem::close(fd);
        }
    }
}

pub fn init() {
    CLI.lock().working_dir = String::from("/");
    println!("CLI: use cd <path>, mkdir <path>, mkfile <path>, edit <path> <content>,");
    println!("rm <path>, show <path>, '/' might be bound to the '#' Key  ");
    print!("/>");
}