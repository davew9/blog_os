
use lazy_static::lazy_static;
use core::sync::atomic::{AtomicUsize, Ordering, AtomicBool};
use core::option::Option;
use self::lock::{RWLock};
use core::ptr::null_mut;
use alloc::boxed::Box;
use crate::filesystem::file::{File};

pub mod lock;
pub mod file;

// Maximum number of entries in LOCALFILETABLE for each task
const LOCALFILETABLE_SIZE:usize = 50;
// Maximum number of Filedescriptions in GLOBALFILETABLE
const GLOBALFILETABLE_SIZE: usize = 100;
// Maximum number of Directories in a Directory
const DIRECTORY_NR: usize = 50;

//______________________________ SYSTEM TABLES____________________________________________________//

lazy_static! {
    // Table contains a Global File Table which contains information about all open files
    static ref GLOBALFILETABLE: RWLock<FileTable> = RWLock::new(FileTable::new());

    // Table Contains File Names and the corresponding reference to the first Node of the file
    static ref ROOTDIRECTORYTABLE: RWLock<DirectoryTable> = RWLock::new(DirectoryTable::new());
}

// Local Filetable. Contains Mapping of local Fildescriptor to global Filedescriptor for
// every Task. Each Task has its FileDescriptors in one row of the 2D-Array
// Global File Descriptor is the Content, Local File Descriptor the Position of the Array Field
static mut LOCALFILETABLE: LocalFileTable = LocalFileTable{active_task:0, table: [[255;20];LOCALFILETABLE_SIZE]};


//__________________________FILESYSTEM API________________________________________________________//

// Creates new file and all non existing directories in path
pub fn create(path: &str) {
    if path.len() > 32 {
        println!("Path is too long");
        return
    }
    let mut node;
    let mut root = true;
    // DirectoryTable of current directory
    let mut dir_table = &mut DirectoryTable::new();
    // Root DirectoryTable
    let mut fn_table = ROOTDIRECTORYTABLE.wlock();
    // Split paths in Directory/File strings
    let mut chunks = path.split("/").peekable();

    // Iterate over all Directory/Files of the given Path
    while let Some(chunk) = chunks.next() {
        if chunk.len() > 0 {
            let chunk_u8 = str_to_u8(chunk);
            // CASE: There is a following Directory/File: Current chunk has to represent a directory
            if chunks.peek().is_some() {
                // CASE: It is a first level directory
                // -> Lookup in Global Root DirectoryTable
                if root {
                    node = fn_table.get_mut(&chunk_u8);
                }
                // CASE: It is not a first level directory
                // ->  Lookup in DirectoryTable of previous node
                else {
                    node = dir_table.get_mut(&chunk_u8);
                }

                // CASE: There was no directory/file with this name
                // -> Create the directory at the corresponding level
                if !node.is_some() {
                    let new_node: Box<Node> = Box::new(Node::DirectoryNode(Directory(DirectoryTable::new())));
                    let new_node: *mut Node = Box::into_raw(new_node);
                    if root {
                        fn_table.insert(chunk_u8, new_node);
                    } else {
                        dir_table.insert(chunk_u8, new_node);
                    }
                    node = Some(new_node);
                    println!("Directory '{}' was created", chunk);
                }
                root = false;

                // CASE: There was a node with this name. If its an directory, set this directory
                // as current Directory
                unsafe {
                    match *(node.unwrap()) {
                        Node::DirectoryNode(ref mut d) => { dir_table = &mut d.0 },
                        _ => panic!("'{}' is no directory", chunk)
                    }
                }

                //CASE: There is no following Directory/File: Current chunk represents the file to be created
            } else {
                //CASE: The file is located at first level
                //Create the file and store the pointer in the Root FileDirectoryTable
                if root {
                    if fn_table.exists(&chunk_u8) {
                        println!("'{}' already exists at this place", chunk);
                        return
                    }
                    // Create an Empty File
                    let node: Box<Node> = Box::new(Node::FileNode(File::new()));
                    let node: *mut Node = Box::into_raw(node);
                    fn_table.insert(chunk_u8, node);
                }
                //CASE: The file is not located at first level
                //Create the file and store the pointer in the FileTable of the previous directory
                else {
                    if dir_table.exists(&chunk_u8) {
                        println!("'{}' already exists at this place", chunk);
                        return
                    }
                    let node: Box<Node> = Box::new(Node::FileNode(File::new()));
                    let node: *mut Node = Box::into_raw(node);
                    dir_table.insert(chunk_u8, node);
                }
                println!("File '{}' was created", chunk)
            }
        }
    }
}

// Creates new directory and all non existing directories in path
pub fn create_dir(path: &str) {
    if path.len() > 32 {
        println!("Path is too long");
        return
    }
    let mut node;
    let mut root = true;
    // FileDirectoryTable of current directory
    let mut dir_table = &mut DirectoryTable::new();
    // Root FileDirectoryTable
    let mut fn_table = ROOTDIRECTORYTABLE.wlock();
    // Split paths in Directory/File strings
    let mut chunks = path.split("/").peekable();

    // Iterate over all Directory/Files of the given Path
    while let Some(chunk) = chunks.next() {
        if chunk.len() > 0 {
            let chunk_u8 = str_to_u8(chunk);
            // CASE: There is a following Directory/File: Current chunk has to represent a directory
            if chunks.peek().is_some() {
                // CASE: It is a first level directory
                // -> Lookup in Global Root FileDirectoryTable
                if root {
                    node = fn_table.get_mut(&chunk_u8);
                }
                // CASE: It is not a first level directory
                // ->  Lookup in DirectoryTable of previous node
                else {
                    node = dir_table.get_mut(&chunk_u8);
                }

                // CASE: There was no directory/file with this name
                // -> Create the directory at the corresponding level
                if !node.is_some() {
                    let new_node: Box<Node> = Box::new(Node::DirectoryNode(Directory(DirectoryTable::new())));
                    let new_node: *mut Node = Box::into_raw(new_node);
                    if root {
                        fn_table.insert(chunk_u8, new_node);
                    } else {
                        dir_table.insert(chunk_u8, new_node);
                    }
                    node = Some(new_node);
                    println!("Directory '{}' was created", chunk);
                }
                root = false;

                // CASE: There was a node with this file, but its not a directory node
                unsafe {
                    match *(node.unwrap()) {
                        Node::DirectoryNode(ref mut d) => { dir_table = &mut d.0 },
                        _ => panic!("'{}' is no directory", chunk)
                    }
                }

                //CASE: There is no following Directory/File: Current chunk represents the file to be created
            } else {
                //CASE: The directory is located at first level
                //Create the directory and store the pointer in the Root FileDirectoryTable
                if root {
                    if fn_table.exists(&chunk_u8) {
                        println!("'{}' already exists at this place", chunk);
                        return
                    }
                    // Create an Empty Directory
                    let node: Box<Node> = Box::new(Node::DirectoryNode(Directory(DirectoryTable::new())));
                    let node: *mut Node = Box::into_raw(node);
                    fn_table.insert(chunk_u8, node);
                }
                //CASE: The directory is not located at firs level
                //Create the directory and store the pointer in the FileTable of the previous directory
                else {
                    if dir_table.exists(&chunk_u8) {
                        println!("'{}' already exists at this place", chunk);
                        return
                    }
                    // Create an Empty Directory
                    let node: Box<Node> = Box::new(Node::DirectoryNode(Directory(DirectoryTable::new())));
                    let node: *mut Node = Box::into_raw(node);
                    dir_table.insert(chunk_u8, node);
                }
                println!("Directory '{}' was created", chunk)
            }
        }
    }
}

// Validates the path to a directory (Used by CLI only)
pub fn validate_path(path: &str) -> bool {
    let mut node;
    let mut root = true;
    // FileDirectoryTable of current directory
    let mut dir_table = &mut DirectoryTable::new();
    // Root FileDirectoryTable
    let fn_table = ROOTDIRECTORYTABLE.wlock();
    // CASE: Path is root
    if path == "/"{
        return true;
    }

    // Iterate over substrings of path
    let mut chunks = path.split("/").peekable();
    while let Some(chunk) = chunks.next() {
        if chunk.chars().count() > 0 {
            let chunk_u8 = str_to_u8(chunk);
            // CASE: Last Element of Path hasn't been reached yet
            // -> Validate Path, Reset the FileDirectoryTable of current directory if valid
            //    directory is encountered
            if chunks.peek().is_some() {
                if root {
                    node = fn_table.get_mut(&chunk_u8);
                } else {
                    node = dir_table.get_mut(&chunk_u8);
                }
                if !node.is_some() {
                    println!("Directory: '{}' doesn't exist", chunk);
                    return false;
                }

                unsafe {
                    match *(node.unwrap()) {
                        Node::DirectoryNode(ref mut d) => { dir_table = &mut d.0 },
                        _ => {
                            println!("'{}' is no directory", chunk);
                            return false;
                        }
                    }
                }
                root = false;
            }


            // CASE:  Last Element of Path has been reached
            else {
                // CASE: Element is located at first level
                // -> lookup in Root FileDirectoryTable
                if root {
                    node = fn_table.get_mut(&chunk_u8);
                }
                // CASE: Element is not located at first level
                // -> lookup in FileDirectoryTable of previous DirectoryNOde
                else {
                    node = dir_table.get_mut(&chunk_u8);
                }
                // CASE: Directory doesn't exist
                if !node.is_some() {
                    println!(" '{}' doesn't exist", chunk);
                    return false;
                }

                unsafe {
                    match  &*(node.unwrap()) {
                        // CASE: Last Substring represents valid Directory
                        Node::DirectoryNode(ref _d) => {
                            return true;
                        },

                        // CASE: Last Substring refers to a File
                        _ => {
                            println!(" '{}' is a file", chunk);
                            return false;
                        }
                    }
                }
            }
        }
    }
    println!("Invalid Path");
    return false;
}

// Deletes the Directory/File specified by the last part of the path string
pub fn delete(path: &str) {
    let path_u8 = str_to_u8(path);
    let mut node;
    let mut root = true;
    // FileDirectoryTable of current directory
    let mut dir_table = &mut DirectoryTable::new();
    // READ LOCK GLOBALFILETABLE, to avoid Deadlock.
    let f_table = GLOBALFILETABLE.rlock();
    // Root FileDirectoryTable
    let mut fn_table = ROOTDIRECTORYTABLE.wlock();

    let mut chunks = path.split("/").peekable();
    while let Some(chunk) = chunks.next() {
        if chunk.len() > 0 {
            let chunk_u8 = str_to_u8(chunk);
            // CASE: Last Element of Path hasn't been reached yet
            // -> Validate Path, Reset the FileDirectoryTable of current directory if valid
            //    directory is encountered
            if chunks.peek().is_some() {
                if root {
                    node = fn_table.get_mut(&chunk_u8);
                } else {
                    node = dir_table.get_mut(&chunk_u8);
                }
                if !node.is_some() {
                    println!("Directory: '{}' doesn't exist", chunk);
                    return
                }

                unsafe {
                    match *(node.unwrap()) {
                        Node::DirectoryNode(ref mut d) => { dir_table = &mut d.0 },
                        _ => {  println!("'{}' is no directory", chunk);
                        return
                        }
                    }
                }
                root = false;
            }

            // CASE:  Last Element of Path (=the Element to delete) has been reached
            else {
                // CASE: Element is located at first level
                // -> lookup in Root FileDirectoryTable
                if root {
                    node = fn_table.get_mut(&chunk_u8);
                }
                // CASE: Element is not located at first level
                // -> lookup in FileDirectoryTable of previous DirectoryNOde
                else {
                    node = dir_table.get_mut(&chunk_u8);
                }
                // CASE: Directory or File doesn't exist
                if !node.is_some() {
                    println!(" '{}' doesn't exist", chunk);
                    return;
                }

                unsafe {
                    match &*(node.unwrap()) {
                        // CASE: Object to delete is directory
                        Node::DirectoryNode(ref d) => {
                            let index = (d.0).0.iter().position(|r| r.name != [0; 32]);
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
        f_table.table[x].open.fetch_add(1, Ordering::SeqCst);
        unsafe {fd_loc = LOCALFILETABLE.add_entry(x)}
        return Some(fd_loc)
    }

    // CASE: The file isn't already open
    // -> Validate Path and open the file corresponding to the last element of the path
    let mut root = true;
    let mut node;
    // FileDirectoryTable of current directory
    let mut dir_table = &DirectoryTable::new();
    // Root FileDirectoryTable
    let fn_table = ROOTDIRECTORYTABLE.rlock();

    let mut chunks = path.split("/").peekable();
    while let Some(chunk) = chunks.next() {
        if chunk.len() > 0 {
            let chunk_u8 = str_to_u8(chunk);
            // CASE: Last Element of Path hasn't been reached yet
            // -> Validate Path,
            if chunks.peek().is_some() {
                // CASE: Current directory is a first level directory
                // ->  Lookup in global ROOTDIRECTORYTABLE
                if root {
                    root = false;
                    node = fn_table.get_mut(&chunk_u8);
                }
                // CASE: Current directory is not a first level directory
                // -> Lookup in DirectoryTable of the previous directory
                else {
                    node = dir_table.get_mut(&chunk_u8);
                }
                // CASE: There is no file at this position with this name
                if !node.is_some() {
                    println!("Directory: '{}' doesn't exist", chunk);
                    return None;
                }

                //Set the FileDirectoryTable of current directory new if valid
                unsafe {
                    match &*(node.unwrap()) {
                        &Node::DirectoryNode(ref d) => { dir_table = &d.0 },
                        _ => {
                            println!("'{}' is no directory", chunk);
                            return None
                        }
                    }
                }
            }
            // CASE: Last Element of Path (= the file to open) has been reached
            else {
                // Get the pointer to the file from the root FileDirectoryTable or from the
                // parent directory
                if root {
                    node = fn_table.get_mut(&chunk_u8);
                } else {
                    node = dir_table.get_mut(&chunk_u8);
                }
                // Release Lock on Directory Table
                drop(fn_table);
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
                    fd_loc = LOCALFILETABLE.add_entry(fd_glob.unwrap())
                }
                return Some(fd_loc);
            }
        }
    }
    println!("Invalid Path");
    return None;
}


// Closes file
pub fn close(fd_loc: usize) {
    let fd_glob;
    // Get FileDescription corresponding to the local FD
    unsafe {fd_glob = LOCALFILETABLE.get_global_fd(fd_loc)};
    // Delete the entry in the Local File Table
    unsafe {LOCALFILETABLE.delete_entry(fd_loc)};
    let mut f_table= GLOBALFILETABLE.wlock();
    f_table.table[fd_glob].open.fetch_sub(1, Ordering::SeqCst);
    // If the file is not open anywhere else (Open==0) remove the entry
    if f_table.table[fd_glob].open.load(Ordering::SeqCst) ==0 {
        f_table.table[fd_glob] = FileDescription::new(null_mut(),[0;32]);
    }
}


// Writes to an file
pub fn write(fd: usize, data: &str, append: bool ) {
    // get FileDescription corresponding to the local FD
    let fd_glob;
    unsafe {fd_glob = LOCALFILETABLE.get_global_fd(fd)}
    let mut guard = GLOBALFILETABLE.wlock();
    // Lock the entry in the GLOBALFILETABLE and get the pointer to the data
    let node = guard.get_w_access(fd_glob);
    if !node.is_some() {
        println!("File is locked");
        return;
    }
    // Call write function of FileNode
    unsafe {
        match *(node.unwrap()) {
            Node::FileNode(ref mut f) => {
                if !append {f.write(data) }
                else {f.append(data)}},
            _ => panic!("UNEXPECTED DIRECTORY")}
    }
    // Release Lock of the file
    guard.return_w_access(fd_glob);
}


// reads from file
pub fn read(fd: usize, offset: usize) -> Option<[u8;1024]>{
    // get FileDescription corresponding to the local FD
    // Lock the corresponding Entry as READ
    let fd_glob;
    unsafe {fd_glob = LOCALFILETABLE.table[LOCALFILETABLE.active_task as usize][fd];}
    let mut guard = GLOBALFILETABLE.rlock();
    let node = guard.get_r_access(fd_glob);

    if !node.is_some() {
        println!("No read possible. File is locked");
        return None;
    }

    // Get data from node and return data
    let data;
    unsafe {
        match *node.unwrap() {
            Node::FileNode(ref mut f) => {
                let data_vector = f.read(offset, offset+1);
                if data_vector.len() > 0 {
                    data = data_vector[0];
                }
                else {
                    // Release Lock of the file
                    guard.return_r_access(fd_glob);
                    return None;
                }
            },
            _ => panic!("UNEXPECTED DIRECTORY")
        }
    }
    // Release Lock of the file
    guard.return_r_access(fd_glob);
    Some(data) }

// is called by the Executor when the task is switched
// Tells which (column of the) LOCALFILETABLE has to be used
pub unsafe fn set_active_task (id: u64) {
    LOCALFILETABLE.active_task = id;
}


//______________________STRUCTS AND STRUCT IMPLEMENTATIONS________________________________________//

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

unsafe impl Send for FileDescription{}

struct DirectoryTable([DirectoryEntry;DIRECTORY_NR]);

impl DirectoryTable{
    pub const fn new() -> DirectoryTable {
        DirectoryTable([DirectoryEntry{node: null_mut(), name:[0;32] }; DIRECTORY_NR])
    }

    fn insert(&mut self, name: [u8;32], node: *mut Node) {
        //search for empty entry
        let index = self.0.iter().position(|r| r.name ==[0;32]).unwrap();
        // Replace empty entry with new entry
        self.0[index] = DirectoryEntry{node: node, name: name}
    }

    fn get_mut(&self, name: &[u8;32]) -> Option<*mut Node>{
        // search for path
        let index = self.0.iter().position(|r| r.name == *name);
        // return pointer to node
        return match index {
            Some(x) => Some(self.0[x].node),
            None => None,
        }
    }

    fn delete(&mut self, name: &[u8;32]) {
        // Replaces entry with an blank entry
        let index = self.0.iter().position(|r| r.name == *name).unwrap();
        self.0[index] = DirectoryEntry{node: null_mut(), name: [0;32]}
    }

    fn exists(&mut self, name: &[u8;32] ) -> bool {
        let index = self.0.iter().position(|r| r.name == *name);
        match index {
            Some(_i) => return true,
            _ => false
        }
    }
}

struct DirectoryEntry{
    name: [u8;32],
    node: *mut Node
}

unsafe impl Send for DirectoryEntry{}

struct FileTable{
    table: [FileDescription;GLOBALFILETABLE_SIZE]
}

impl FileTable {
    pub const fn new() -> FileTable
    {
        FileTable
        {
            table: [FileDescription::new(null_mut(),[0;32]);GLOBALFILETABLE_SIZE]
        }
    }

    // Returns pointer to node, if entry isn't locked by Write
    fn get_r_access(&mut self, fd: usize) -> Option<*mut Node> {
        // Make sure there is no WRITE on the file
        if self.table[fd].writes.load(Ordering::SeqCst) == true
        {
            return None
        }

        // increment the read semaphore
        self.table[fd].reads.fetch_add(1, Ordering::SeqCst);

        // make sure no write locks have occured in the mean time.
        if self.table[fd].writes.load(Ordering::SeqCst) == true
        {
            self.table[fd].reads.fetch_sub(1, Ordering::SeqCst);
            return None;
        }
        Some(self.table[fd].node)
    }

    // Removes the Read flag from FileDescription
    fn return_r_access(&mut self, fd: usize) {
        self.table[fd].reads.fetch_sub(1, Ordering::SeqCst);
    }

    // Returns pointer to node, if entry isn't locked by Write or READ
    fn get_w_access(&mut self, fd: usize) -> Option<*mut Node> {
        // Try to lock read
        if self.table[fd].writes.compare_and_swap(false, true, Ordering::SeqCst) != false
        {
            return None
        }
        // Make sure their are no writes
        if self.table[fd].reads.load(Ordering::SeqCst) != 0
        {
            self.table[fd].writes.store(false, Ordering::SeqCst);
            return None
        }

        Some(self.table[fd].node)
    }

    // Removes the Write flag from FileDescription
    fn return_w_access(&mut self, fd: usize) {
        self.table[fd].writes.store(false, Ordering::SeqCst);
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

struct LocalFileTable{
    table: [[usize;20];LOCALFILETABLE_SIZE],
    active_task: u64
}

impl LocalFileTable {
    fn add_entry(&mut self, global_file_desc: usize) -> usize
    {
        //search for empty file descriptor position
        //255 represents blank entry
        let index = self.table[self.active_task as usize].iter().position(|r| *r == 255);
        //set local entry and return index
        match index {
            Some(index) => {
                self.table[self.active_task as usize][index] = global_file_desc;
                index },
            None => {
                println!("ERROR: Too many files open");
                return 0},
        }
    }

    // Returns the matching global FD to the given local FD
    fn get_global_fd(&mut self, fd_loc: usize) -> usize {
        self.table[self.active_task as usize][fd_loc]
    }

    // Removes Entry from local FileTable
    fn delete_entry(&mut self, local_file_desc: usize) {
        self.table[self.active_task as usize][local_file_desc] = 255;
    }

}

pub struct Directory(DirectoryTable);

pub enum Node{
    FileNode(File),
    DirectoryNode(Directory),
}

impl Node {
    fn is_directory(&self) -> bool {
        match self {
            Node::DirectoryNode(_d) => true,
            _ => false
        }
    }
}

//________________________HELPER FUNCTIONS AND FUNCTIONS FOR TESTING______________________________//

// Transforms string slice to u8 Array.
// Used to save filename (&str)
fn str_to_u8(s: &str) -> [u8;32]{
    let mut data: [u8;32] = [0;32];
    let mut i = 0;
    for byte in s.bytes() {
        if i<32 {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e => data[i] = byte,
                // not part of printable ASCII range
                _ =>data[i] = 0xfe,
            }
            i+=1;
        }
    }
    data
}

// Transforms string slice to u8 Array of fixed size.
// Used to save file content (&str) as bytes
fn str_to_file(s: &str) -> [u8;1024]{
    let mut data: [u8;1024] = [0;1024];
    let mut i = 0;
    for byte in s.bytes() {
        if i<1024 {
            data[i] = byte;
            i += 1;
        }
    }
    data
}

// Only for testing of the lock function
pub fn w_lock(fd: usize, lock: bool) {
    let fd_glob;
    unsafe {fd_glob = LOCALFILETABLE.table[LOCALFILETABLE.active_task as usize][fd];}
    let mut guard = GLOBALFILETABLE.rlock();
    if lock {
        guard.get_w_access(fd_glob);
    }
    else {
        guard.return_w_access(fd_glob);
    }
}

// Only for testing of the lock function
pub fn r_lock(fd: usize, lock: bool) {
    let fd_glob;
    unsafe {fd_glob = LOCALFILETABLE.table[LOCALFILETABLE.active_task as usize][fd];}
    let mut guard = GLOBALFILETABLE.rlock();
    if lock {
        guard.get_r_access(fd_glob);
    }
    else {
        guard.return_r_access(fd_glob);
    }
}

// Only for testing
pub fn init(){

}