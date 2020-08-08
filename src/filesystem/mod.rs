
use lazy_static::lazy_static;
//use alloc::{collections::BTreeMap};
use core::sync::atomic::{AtomicUsize, Ordering, AtomicBool};
use core::option::Option;
use self::lock::{RWLock};
use crate::vga_buffer::print_bytes;
use core::ptr::null_mut;
use alloc::boxed::Box;


pub mod lock;
pub mod file;



lazy_static! {

    /// Table contains a Global File Table which contains information about all open files
    static ref GLOBALFILETABLE: RWLock<FileTable> = RWLock::new(FileTable::new());

    /// Table Contains File Names and the corresponding reference to the first Node of the file
    static ref FILENAMETABLE: RWLock<NameTable> = RWLock::new(NameTable::new());
}

/// Local Filetable. Contains Mapping of local Fildescriptor to global Filedescriptor for
/// every Task. Each Task has its FileDescriptors in one row of the 2D-Array
/// Global File Descriptor is the Content, Local File Descriptor the Position of the Array Field
/// ToDo sehr unschön
static mut LOCALFILETABLE: LocalFileTable = LocalFileTable{active_task:0, table: [[255;20];20]};

// bisher nur zu testzwecken, wird mit blog_os::init aufgerufen.
pub fn init(){

}

// Transformiert string slice zu u8 Array.
// Wird verwendet um Filenamen (&str) abzuspeichern
fn str_to_u8(s: &str) -> [u8;32]{
    let mut data: [u8;32] = [0;32];
    let mut i = 0;
    for byte in s.bytes() {
        if i<32 {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => data[i] = byte,
                // not part of printable ASCII range
                _ =>data[i] = 0xfe,
            }
            i+=1;
        }
    }
    data
}

// Transforms string slice to u8 Array of fixed size.
// Ohne Größenbeschränkung möglich?
// Wird verwendet um Fileinhalt (&str) als Byte abzuspeichern
fn str_to_file(s: &str) -> [u8;1024]{
    let mut data: [u8;1024] = [0;1024];
    let mut i = 0;
    for byte in s.bytes() {
        match byte {
            // printable ASCII byte or newline
            0x20..=0x7e | b'\n' => data[i] = byte,
            // not part of printable ASCII range
            _ =>data[i] = 0xfe,
        }
        i+=1;
    }
    data
}


// Creates new file and all non existing directories in path
pub fn create(path: &str) {
    let mut node;
    let mut root = true;
    // FileNameTable of current directory
    let mut dir_table = &mut NameTable::new();
    // Root FileNameTable
    let mut fn_table = FILENAMETABLE.wlock();
    // Split paths in Directory/File strings
    let mut chunks = path.split("/").peekable();

    // Iterate over all Directory/Files of the given Path
    while let Some(chunk) = chunks.next() {
        let chunk_u8 = str_to_u8(chunk);
        // CASE: There is a following Directory/File: Current chunk has to represent a directory
        if chunks.peek().is_some() {
            // CASE: It is a first level directory
            // -> Lookup in Global Root FileNameTable
            if root {
                node = fn_table.get_mut(&chunk_u8);
            }
            // CASE: It is not a first level directory
            // ->  Lookup in NameTable of previous node
            else {
                node = dir_table.get_mut(&chunk_u8);
            }

            // CASE: There was no directory/file with this name
            // -> Create the directory at the corresponding level
            if !node.is_some() {
                let new_node: Box<Node> = Box::new(Node::Directory(DirectoryNode(NameTable::new())));
                let new_node: *mut Node = Box::into_raw(new_node);
                if root {
                    fn_table.insert(chunk_u8, new_node);
                }
                else {
                    dir_table.insert(chunk_u8, new_node);
                }
                node = Some(new_node);
                println!("Directory '{}' was created", chunk);
            }
            root = false;

            // CASE: There was a node with this file, but its not a directory node
            unsafe {
                match  *(node.unwrap()) {
                    Node::Directory(ref mut d) => {dir_table = &mut d.0},
                    _ => panic!("'{}' is no directory", chunk)
                }
            }

            //CASE: There is no following Directory/File: Current chunk represents the file to be created
        } else {
            // Create an Empty File
            let node: Box<Node> = Box::new(Node::File(FileNode([0;1024])));
            let node: *mut Node = Box::into_raw(node);
            //CASE: The file is located at first level
            //Create the file and store the pointer in the Root FileNameTable
            if root {
                fn_table.insert(chunk_u8, node);
            }
            //CASE: The file is not located at firs level
            //Create the file and store the pointer in the FileTable of the previous directory
            else {
                dir_table.insert(chunk_u8, node);
            }
            println!("File '{}' was created", chunk)

        }
    }
}

// Creates new directory and all non existing directorys in path
// TODO Viel Code identisch zu create, kann man vielleicht in funktion kapseln
pub fn create_dir(path: &str) {
    let mut node;
    let mut root = true;
    // FileNameTable of current directory
    let mut dir_table = &mut NameTable::new();
    // Root FileNameTable
    let mut fn_table = FILENAMETABLE.wlock();
    // Split paths in Directory/File strings
    let mut chunks = path.split("/").peekable();

    // Iterate over all Directory/Files of the given Path
    while let Some(chunk) = chunks.next() {
        let chunk_u8 = str_to_u8(chunk);
        // CASE: There is a following Directory/File: Current chunk has to represent a directory
        if chunks.peek().is_some() {
            // CASE: It is a first level directory
            // -> Lookup in Global Root FileNameTable
            if root {
                node = fn_table.get_mut(&chunk_u8);
            }
            // CASE: It is not a first level directory
            // ->  Lookup in NameTable of previous node
            else {
                node = dir_table.get_mut(&chunk_u8);
            }

            // CASE: There was no directory/file with this name
            // -> Create the directory at the corresponding level
            if !node.is_some() {
                let new_node: Box<Node> = Box::new(Node::Directory(DirectoryNode(NameTable::new())));
                let new_node: *mut Node = Box::into_raw(new_node);
                if root {
                    fn_table.insert(chunk_u8, new_node);
                }
                else {
                    dir_table.insert(chunk_u8, new_node);
                }
                node = Some(new_node);
                println!("Directory '{}' was created", chunk);
            }
            root = false;

            // CASE: There was a node with this file, but its not a directory node
            unsafe {
                match  *(node.unwrap()) {
                    Node::Directory(ref mut d) => {dir_table = &mut d.0},
                    _ => panic!("'{}' is no directory", chunk)
                }
            }

            //CASE: There is no following Directory/File: Current chunk represents the file to be created
        } else {
            // Create an Empty Directory
            let node: Box<Node> = Box::new(Node::Directory(DirectoryNode(NameTable::new())));
            let node: *mut Node = Box::into_raw(node);
            //CASE: The directory is located at first level
            //Create the directory and store the pointer in the Root FileNameTable
            if root {
                fn_table.insert(chunk_u8, node);
            }
            //CASE: The directory is not located at firs level
            //Create the directory and store the pointer in the FileTable of the previous directory
            else {
                dir_table.insert(chunk_u8, node);
            }
            println!("Directory '{}' has been created", chunk)

        }
    }
}

// Deletes the Directory/File specified by the last part of the path string
pub fn delete(path: &str) {
    let path_u8 = str_to_u8(path);
    let mut node;
    let mut root = true;
    // FileNameTable of current directory
    let mut dir_table = &mut NameTable::new();
    // Root FileNameTable
    let mut fn_table = FILENAMETABLE.wlock();

    let mut chunks = path.split("/").peekable();
    while let Some(chunk) = chunks.next() {
        let chunk_u8 = str_to_u8(chunk);
        // CASE: Last Element of Path hasn't been reached yet
        // -> Validate Path, Reset the FileNameTable of current directory if valid
        //    directory is encountered
        if chunks.peek().is_some() {
            if root {
                node = fn_table.get_mut(&chunk_u8);
            } else {
                node = dir_table.get_mut(&chunk_u8);
            }
            if !node.is_some() {
                println!("Directory: '{}' doesn't exist", chunk)
            }

            unsafe {
                match *(node.unwrap()) {
                    Node::Directory(ref mut d) => { dir_table = &mut d.0 },
                    _ => println!("'{}' is no directory", chunk)
                }
            }
            root = false;
        }


        // CASE:  Last Element of Path (=the Element to delete) has been reached
        else {
            // CASE: Element is located at first level
            // -> lookup in Root FileNameTable
            if root {
                node = fn_table.get_mut(&chunk_u8);
            }
            // CASE: Element is not located at first level
            // -> lookup in FileNameTable of previous DirectoryNOde
            else {
                node = dir_table.get_mut(&chunk_u8);
            }
            // CASE: Directory or File doesn't exist
            if !node.is_some() {
                println!(" '{}' doesn't exist", chunk);
                return;
            }

            unsafe {
                match  &*(node.unwrap()) {
                    // CASE: Object to delete is directory
                    Node::Directory(ref d) => {
                        let index = (d.0).0.iter().position(|r| r.path != [0; 32]);
                        // CASE: Directory is empty
                        // -> Delete Directory
                        if !index.is_some() {
                            drop(Box::from_raw(node.unwrap()));
                            if root { fn_table.delete(&chunk_u8); } else { dir_table.delete(&chunk_u8) }
                            println!("Directory '{}' was deleted", chunk);
                            // CASE: Directory is not empty
                        } else { println!("Directory '{}' contains Files and cannot be deleted", chunk) }
                    },
                    // CASE: Object to delete is file
                    _ => {
                        // Check if file is open
                        let f_table = GLOBALFILETABLE.rlock();
                        let fd_glob = f_table.table.iter().position(|r| r.name == path_u8);
                        // CASE: File is not open
                        // -> Delete File
                        if !fd_glob.is_some() {
                            drop(Box::from_raw(node.unwrap()));
                            if root { fn_table.delete(&chunk_u8); } else { dir_table.delete(&chunk_u8) }
                            println!("File '{}' was deleted", chunk);
                            // CASE: File is open
                        } else {
                            println!("File '{}' is currently open and cannot be deleted", chunk)
                        }
                    }
                }
            }
        }
    }
}

// Opens File, returns Option<File Descriptor>
pub fn open(path: &str) -> Option<usize> {
    // Lock GLOBALFILETABLE and check if file is already open
    // If its open there has to be an entry with the corresponding filename
    let file_name_u8 = str_to_u8(path);
    let fd_loc;
    let mut f_table = GLOBALFILETABLE.wlock();
    let mut fd_glob = f_table.table.iter().position(|r| r.name == file_name_u8);
    // CASE: A global FD for the file exists -> File is already open
    // -> Save global FD in LOCALFILETABLE
    // -> Return the corresponding Local File Descriptor
    if let Some(x) = fd_glob {
        unsafe {fd_loc = LOCALFILETABLE.add_entry(x)}
        return Some(fd_loc)
    }

    // CASE: The file isn't already open
    // -> Validate Path and open the file corresponding to the last element of the path
    let mut root = true;
    let mut node;
    // FileNameTable of current directory
    let mut dir_table = &NameTable::new();
    // Root FileNameTable
    let fn_table = FILENAMETABLE.rlock();

    let mut chunks = path.split("/").peekable();
    while let Some(chunk) = chunks.next() {

        let chunk_u8 = str_to_u8(chunk);
        // CASE: Last Element of Path hasn't been reached yet
        // -> Validate Path,
        if chunks.peek().is_some() {
            // CASE: Current directory is a first level directory
            // ->  Lookup in global FILENAMETABLE
            if root {
                root = false;
                node = fn_table.get_mut(&chunk_u8);
            }
            // CASE: Current directory is not a firs level directory
            // -> Lookup in NameTable of the previous directory
            else {
                node = dir_table.get_mut(&chunk_u8);
            }
            // CASE: There is no file at this position with this name
            if !node.is_some() {
                println!("Directory: '{}' doesn't exist", chunk);
                return None;
            }

            //Set the FileNameTable of current directory new if valid
            unsafe {
                match  &*(node.unwrap()) {
                    &Node::Directory(ref d) => {dir_table = &d.0},
                    _=> {
                        println!("'{}' is no directory", chunk);
                        return None
                    }
                }
            }

        }
        // CASE: Last Element of Path (= the file to open) has been reached
        else {
            // Get the pointer to the file from the root FileNameTable or from the
            // parent directory
            if root {
                node = fn_table.get_mut(&chunk_u8);
            }
            else {
                node = dir_table.get_mut(&chunk_u8);
            }
            if !node.is_some() {
                println!("'{}' doesn't exist", chunk);
                return None
            }


            unsafe {
                if (*(node).unwrap()).is_directory() {
                    println!("'{}' is a directory", chunk);
                    return None
                }
                // Add File Description in GlobalFileTable
                fd_glob = f_table.add_file_description(node.unwrap(), file_name_u8);
                // Add Global File Descriptor to LocalFileTable
                fd_loc = LOCALFILETABLE.add_entry(fd_glob.unwrap())}
            return Some(fd_loc);
        }
    }
    unreachable!("FILESYSTEM FAILURE");
}


// Removes the FD from the LOCALFILETABLE
// Removes the FD from the GLOBALFILETABLE if the file isn't open somewhere else
pub fn close(fd_loc: usize) {
    let fd_glob;
    unsafe {fd_glob = LOCALFILETABLE.get_global_fd(fd_loc)};
    unsafe {LOCALFILETABLE.delete_entry(fd_loc)};
    let mut f_table= GLOBALFILETABLE.wlock();
    f_table.table[fd_glob].open.fetch_sub(1, Ordering::SeqCst);
    // If the file is not open anywhere else (Open==0) remove the entry
    if f_table.table[fd_glob].open.load(Ordering::Relaxed) ==0 {
        f_table.table[fd_glob] = FileDescription::new(null_mut(),[0;32]);
    }
}


// Writes to an file
//TODO Offset implementieren
pub fn write(fd: usize, _offset: usize, data: &str ) {
    // search file in GLOBALFILETABLE
    let data_u8 = str_to_file(data);
    let fd_glob;
    unsafe {fd_glob = LOCALFILETABLE.get_global_fd(fd)}
    let mut guard = GLOBALFILETABLE.wlock();
    // Lock the entry in the GLOBALFILETABLE and get the pointer to the data
    let node = guard.get_w_access(fd_glob);
    if !node.is_some() {
        println!("File is locked");
    }
    // Write to the pointer
    unsafe {
        match *(node.unwrap()) {
            Node::File(ref mut f) => f.0 = data_u8,
            _ => panic!("UNEXPECTED DIRECTORY")}
    }
    // Release Lock of the file
    guard.return_w_access(fd_glob);
}


// reads from file
//TODO Offset implementieren
pub fn read(fd: usize, _offset: usize) -> Option<[u8;1024]>{
    // get pointer to Node from GLOBALFILETABLE
    // Lock the corresponding Entry as READ
    let fd_glob;
    unsafe {fd_glob = LOCALFILETABLE.table[LOCALFILETABLE.active_task as usize][fd];}
    let mut guard = GLOBALFILETABLE.rlock();
    let node = guard.get_r_access(fd_glob);

    if !node.is_some() {
        println!("No read possible File is locked");
        return None;
    }

    // Get data from node and return
    let data;
    unsafe {
        match *node.unwrap() {
            Node::File(ref f) => {
                data = f.0
            },
            _ => panic!("UNEXPECTED DIRECTORY")
        } }
    guard.return_r_access(fd_glob);
    Some(data)
}

// is called by the Executor when the task is switched
// Tells which (column of the) LOCALFILETABLE has to be used
// TODO Sehr Unschön. Volatile? Atomic? nötig.
pub unsafe fn set_active_task (id: u64) {
    LOCALFILETABLE.active_task = id;
}


// Entry in GLOBALFILETABLE
struct FileDescription {
    node:  *mut Node,
    name: [u8;32],
    reads: AtomicUsize,
    open: AtomicUsize,
    writes: AtomicBool,
}


impl FileDescription {
    pub const fn new(node: *mut Node, path:[u8;32]) -> FileDescription
    {
        FileDescription
        {
            node,
            name: path,
            reads: AtomicUsize::new(0),
            open: AtomicUsize::new(1),
            writes: AtomicBool::new(false)
        }
    }
}

pub struct NameTable([NameEntry;100]);

pub struct NameEntry{
    path: [u8;32],
    node: *mut Node
}



impl NameTable{
    pub const fn new() -> NameTable {
        NameTable([NameEntry{node: null_mut(), path:[0;32] }; 100])
    }

    fn insert(&mut self, path: [u8;32], node: *mut Node) {
        //search for empty entry
        let index = self.0.iter().position(|r| r.path ==[0;32]).unwrap();
        // Replace empty entry with new entry
        self.0[index] = NameEntry{node: node, path: path}
    }

    fn get_mut(&self, path: &[u8;32]) -> Option<*mut Node>{
        // search for path
        let index = self.0.iter().position(|r| r.path == *path);
        // return pointer to node
        return match index {
            Some(x) => Some(self.0[x].node),
            None => None,
        }
    }

    fn delete(&mut self, path: &[u8;32]) {
        let index = self.0.iter().position(|r| r.path == *path).unwrap();
        self.0[index] = NameEntry{node: null_mut(), path: [0;32]}
    }
}


// 100 Einträge maximal in der GLOBALFILETABLE
pub struct FileTable{
    table: [FileDescription;100]
}

//20 Einträge maximal in der LocalfileTable
pub struct LocalFileTable{
    table: [[usize;20];20],
    active_task: u64
}


impl LocalFileTable {

    fn add_entry(&mut self, global_file_desc: usize) -> usize
    {
        //search for empty file descriptor position
        let index = self.table[self.active_task as usize].iter().position(|r| *r == 255).unwrap();
        //set local entry and return index
        self.table[self.active_task as usize][index] = global_file_desc;
        index
    }

    // Returns the matching global FD to the given local FD
    fn get_global_fd(&mut self, fd_loc: usize) -> usize {
        self.table[self.active_task as usize][fd_loc]
    }

    // Removes Entry from local FileTable
    fn delete_entry(&mut self, local_file_desc: usize) {
        self.table[self.active_task as usize][local_file_desc] = 0;
    }

}

// Manche kann man vielleicht streichen
unsafe impl Send for LocalFileTable {}
unsafe impl Sync for LocalFileTable {}
unsafe impl Send for FileDescription{}
unsafe impl Sync for NameEntry{}
unsafe impl Send for NameEntry{}

// File Table, initialized with null pointer and invalid Name("0")
impl FileTable {
    pub const fn new() -> FileTable
    {
        FileTable
        {
            table: [FileDescription::new(null_mut(),[0;32]);100]
        }
    }

    // Returns pointer to node, if entry isn't locked by Write
    fn get_r_access(&mut self, fd: usize) -> Option<*mut Node> {
        // Make sure there is no WRITE on the file
        if self.table[fd].writes.load(Ordering::Acquire) == true
        {
            return None
        }

        // increment the read semaphore
        self.table[fd].reads.fetch_add(1, Ordering::Acquire);

        // make sure no write locks have occured in the mean time.
        if self.table[fd].writes.load(Ordering::Acquire) == true
        {
            self.table[fd].reads.fetch_sub(1, Ordering::Acquire);
            return None;
        }
        Some(self.table[fd].node)
    }

    // Removes the Read flag from FileDescription
    fn return_r_access(&mut self, fd: usize) {
        self.table[fd].reads.fetch_sub(1, Ordering::Acquire);
    }

    // Returns pointer to node, if entry isn't locked by Write or READ
    fn get_w_access(&mut self, fd: usize) -> Option<*mut Node> {
        // Try to lock read
        if self.table[fd].writes.compare_and_swap(false, true, Ordering::Acquire) != false
        {
            return None
        }
        // Make sure their are no writes
        if self.table[fd].reads.load(Ordering::SeqCst) != 0
        {
            self.table[fd].writes.store(false, Ordering::Relaxed);
            return None
        }

        Some(self.table[fd].node)
    }

    // Removes the Write flag from FileDescription
    fn return_w_access(&mut self, fd: usize) {
        self.table[fd].writes.store(false, Ordering::Relaxed);
    }

    fn add_file_description(&mut self, node: *mut Node, file_name : [u8;32]) -> Option<usize>
    {
        //search for empty file descriptor position
        let fd_glob = self.table.iter().position(|r| r.name ==[0;32]);
        // Set a new entry at this position and return the index as FileDescriptor
        self.table[fd_glob.unwrap()] = FileDescription::new(node, file_name);
        fd_glob
    }

}

// ToDo Enthält eigentliche Daten.
// Platzhalter: Metadaten, Dynamisch Größe, Nachfolger Nodes
#[derive( Clone, Copy)]
pub struct FileNode([u8;1024]);

pub struct DirectoryNode(NameTable);

pub enum Node{
    File(FileNode),
    Directory(DirectoryNode),
}

impl Node {
    fn is_directory(&self) -> bool {
        match self {
            Node::Directory(_d) => true,
            _ => false
        }
    }
}