
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


// Creates new file
pub fn create(file_name: &str) {
    let file_name_u8 = str_to_u8(file_name);
    //TODO: SpeicherNode ohne feste Größe?

    // Möglichkeit um Heap Speicher zu allokieren ohne sich Gedanken um Memory Größe/Layout machen zu müssen:
    // Node is initialized in Box<>: Allocated in Heap without manual definition of memory size/
    // alignment. Then Node is casted into raw-pointer. Memory is not freed until
    // Node is casted back in Box<> and dropped
    let node: Box<FileNode> = Box::new(FileNode([0;1024]));
    let node: *mut FileNode = Box::into_raw(node);

    // Node and Filename are saved in FILENAMETABLE
    FILENAMETABLE.wlock().insert(file_name_u8, node);
}

// Opens File, returns File Descriptor
pub fn open(file_name: &str) -> usize {
    // Lock GLOBALFILETABLE and check if file is already open
    // If its open there has to be an entry with the corresponding filename
    let mut f_table = GLOBALFILETABLE.wlock();
    let file_name_u8 = str_to_u8(file_name);
    let fd_loc;
    let mut fd_glob = f_table.table.iter().position(|r| r.name == file_name_u8);
    // If global FD for the file already exists, save global FD in LOCALFILETABLE
    // Return the corresponding Local File Descriptor
    if let Some(x) = fd_glob {
        unsafe {fd_loc = LOCALFILETABLE.add_entry(x)}
        return fd_loc
    }

    // If FD for the file doesn't exist:
    // Search pointer to node by filename in FILENAMETABLE
    let mut fn_table = FILENAMETABLE.rlock();
    let node = fn_table.get_mut(&file_name_u8);
    // Create a new entry in the Globalfiletable and the Localfiletable
    if let Some(x) = node {
        // let copy = x.clone();
        //fd_glob = f_table.set_entry(node, file_name_u8);
        fd_glob = f_table.add_file_description(x, file_name_u8);
        unsafe {fd_loc = LOCALFILETABLE.add_entry(fd_glob.unwrap())}

        return fd_loc
    }
    panic!("File doesn't exist")
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
    unsafe{(*(node.unwrap())).0 = data_u8;}
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
    let data = unsafe{(*(node.unwrap())).0};
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
    node:  *mut FileNode,
    name: [u8;32],
    reads: AtomicUsize,
    open: AtomicUsize,
    writes: AtomicBool,
}


impl FileDescription {
    pub const fn new(node: *mut FileNode, file_name:[u8;32]) -> FileDescription
    {
        FileDescription
        {
            node: node,
            name: file_name,
            reads: AtomicUsize::new(0),
            open: AtomicUsize::new(1),
            writes: AtomicBool::new(false)
        }
    }
}

pub struct NameTable([NameEntry;100]);

pub struct NameEntry{
    path: [u8;32],
    node: *mut FileNode
}



impl NameTable{
    pub const fn new() -> NameTable {
        NameTable([NameEntry{node: null_mut(), path:[0;32] }; 100])
    }
    fn insert(&mut self, path: [u8;32], node: *mut FileNode) {
        //search for empty entry
        let index = self.0.iter().position(|r| r.path ==[0;32]).unwrap();
        // Replace empty entry with new entry
        self.0[index] = NameEntry{node: node, path: path}
    }

    fn get_mut(&mut self, path: &[u8;32]) -> Option<*mut FileNode>{
        // search for path
        let index = self.0.iter().position(|r| r.path == *path);
        // return pointer to node
        return match index {
            Some(x) => Some(self.0[x].node),
            None => None,
        }
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
    fn get_r_access(&mut self, fd: usize) -> Option<*mut FileNode> {
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
    fn get_w_access(&mut self, fd: usize) -> Option<*mut FileNode> {
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

    fn add_file_description(&mut self, node: *mut FileNode, file_name : [u8;32]) -> Option<usize>
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


