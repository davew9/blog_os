
use lazy_static::lazy_static;
//use spin::Mutex;
use alloc::{collections::BTreeMap};
use core::sync::atomic::{AtomicUsize};

use self::lock::RWLock;

pub mod lock;
pub mod file;


lazy_static! {
    /// Table Contains File Names and the corresponding reference to the first Node of the file
    static ref FILENAMETABLE: RWLock<BTreeMap<[u8;32], FileNode>> = RWLock::new(BTreeMap::new());

    /// Table contains a Global File Table which contains information about all open files
    static ref GLOBALFILETABLE: RWLock<FileTable> = RWLock::new(FileTable::new());
 }

// bisher nur zu testzwecken, wird mit blog_os::init aufgerufen.
pub fn init(){
    let mut res = GLOBALFILETABLE.wlock();
    let file_name_u8 = str_to_u8("test2");
    let examplenode = FileNode{chars:11};
    res.table[2] = FileEntry::new(examplenode, file_name_u8);
}

// Transformiert string slice zu u8 Array. Eventuell könnte man auch &str speichern?
fn str_to_u8(s: &str) -> [u8;32]{
    let mut data: [u8;32] = [0xfe;32];
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

// Creates new file
pub fn create(file_name: &str) {
    let file_name_u8 = str_to_u8(file_name);
    let node = FileNode { chars: 5 };
    FILENAMETABLE.wlock().insert(file_name_u8, node);
    //TODO: Tatsächlich Speicher Allokieren
    //Soll File Descriptor zurück gelierfert werden?
}

// Opens File, returns File Descriptor
pub fn open(file_name: &str) -> usize {
    // Lock GLOBALFILETABLE and check if file is already open
    // return File Descriptor
    let mut res = GLOBALFILETABLE.wlock();
    let file_name_u8 = str_to_u8(file_name);
    let index = res.table.iter().position(|r| r.name == file_name_u8);
    if let Some(x) = index {
        return x
    }
    //TODO Falls nicht: File öffnen: Eintrag in GLOBALFILETABLE
    let examplenode = FileNode{chars:8};
    res.table[0] = FileEntry::new(examplenode, file_name_u8);
    0

}


// Entry in GLOBALFILETABLE
struct FileEntry<> {
    node:  FileNode, //TODO ersetzen mit Referenz auf ersten Node.
    name: [u8;32],
    reads: AtomicUsize,
    writes: AtomicUsize
}


impl FileEntry {
    pub const fn new(_node: FileNode, file_name:[u8;32]) -> FileEntry
    {
        FileEntry
        {
            node: FileNode{chars: 0},
            name: file_name,
            reads: AtomicUsize::new(0),
            writes: AtomicUsize::new(0),
        }
    }
}

// 100 Einträge maximal in der GLOBALFILETABLE
pub struct FileTable{
    table: [FileEntry;100]


}

// File Table, initialized with dummy node and invalid Name("Blank")
impl FileTable {
    pub const fn new() -> FileTable
    {
        FileTable
        {
            table: [FileEntry::new(FileNode{chars:0},[0xfe;32]);100]
        }
    }
}


// ToDo Enthält eigentliche Daten.
// Platzhalter
pub struct FileNode {
    pub chars: u8,
}
