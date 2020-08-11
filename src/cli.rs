use lazy_static::lazy_static;
use spin::Mutex;
use alloc::prelude::v1::{String, ToOwned};
use crate::vga_buffer::print_bytes;
use crate::filesystem;

const BUFFER_SIZE:usize = 800;


lazy_static! {
    static ref CLI: Mutex<Cli> = Mutex::new(Cli::new());
}


struct Cli {
    working_dir: String,
    command: [u8;BUFFER_SIZE],
    cursor_pos: usize,

}

impl Cli {
    pub const fn new() -> Cli {
        Cli {
            command: [0; BUFFER_SIZE],
            working_dir: String::new(),
            cursor_pos: 0
        }
    }

    fn reset(&mut self) {
        self.command = [0;BUFFER_SIZE];
        self.cursor_pos = 0;
        print!("{}>", self.working_dir);
    }
}

pub fn add_char(char: &str) {
    let mut cli;
    if char == "\n" {
        parse_command();
        return
    }
    else if char == "\x08" {
        cli = CLI.lock();
        print!("\n");
        cli.reset();
        return
    }

    cli = CLI.lock();
    let cursor = cli.cursor_pos;
    cli.command[cursor] = (*char).as_bytes()[0];
    cli.cursor_pos += 1;
    if cli.cursor_pos == BUFFER_SIZE {
        cli.command = [0;BUFFER_SIZE];
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
        path = wd.trim_end_matches("/").to_owned() + "/" + argument.trim_start();
    }

    if operation == "cd" { if filesystem::validate_path(&path)  {
        cli.working_dir = path;
    }
    }

    else if operation == "mkdir" {mkdir(&path)}
    else if operation == "show" {show(&path, &argument2)}
    else if operation == "rm" {rm(&path)}
    else if operation == "mkfile" {mkfile(&path)}
    else if operation == "edit" {edit(&path, &argument2)}
    else if operation == "apd" {append(&path, &argument2)}
    else if operation == "wlock" {wlock(&path, &argument2)}
    else if operation == "rlock" {rlock(&path, &argument2)}
    else {println!("Unknown command")}
    cli.reset();

}


fn show(path: &str, page: &str ) {
    let page = page.trim().parse::<usize>();
    let mut offset = 0;
    if let Result::Ok(nr) = page{
        offset = nr
    }
    let fd = filesystem::open(path);
    match fd {
        None => return,
        Some(fd) => {
            let content = filesystem::read(fd, offset);
            match content {
                Some(content) => print_bytes(&content),
                None => ()
            }
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

fn append(path: &str, content: &str ) {
    let fd = filesystem::open(path);
    match fd {
        None => return,
        Some(fd) => {
            filesystem::write(fd, content, true);
            filesystem::close(fd);
        }
    }
}

fn edit(path: &str, content: &str ) {
    let fd = filesystem::open(path);
    match fd {
        None => return,
        Some(fd) => {
            filesystem::write(fd, content, false);
            filesystem::close(fd);
        }
    }
}

// Only for testing
fn wlock(path: &str, arg: &str){
    let fd = filesystem::open(path);
    match fd {
        None => return,
        Some(fd) => {
            if arg.contains("free") {
                filesystem::w_lock(fd, false);
                println!("'{}': write lock removed", path);
            }
            else {
                filesystem::w_lock(fd, true);
                println!("'{}': write lock set", path);
            }
        }
    }
}

// Only for testing
fn rlock(path: &str, arg: &str){
    let fd = filesystem::open(path);
    match fd {
        None => return,
        Some(fd) => {
            if arg.contains("free") {
                filesystem::r_lock(fd, false);
                println!("'{}': read lock removed", path);
            }
            else {
                filesystem::r_lock(fd, true);
                println!("'{}': read lock set", path);
            }
        }
    }
}



pub fn init() {
    CLI.lock().working_dir = String::from("/");
    println!("CLI: use cd <path>, mkdir <path>, mkfile <path>, edit <path> <content>,");
    println!("rm <path>, show <path> <page>, apd <path> <content>");
    println!("Backspace to abort ,'/' might be bound to the '#' Key");
    print!("/>");
}