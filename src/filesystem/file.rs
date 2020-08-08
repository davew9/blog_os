use alloc::{vec::Vec, boxed::Box};
use crate::filesystem::{str_to_u8, str_to_file};

const BYTE_LENGTH:usize = 1024;

pub struct FileListNode {
    size: usize,
    data: [u8;BYTE_LENGTH], // 1024 = 1 Byte
    next: Option<&'static mut FileListNode>
}

pub struct File{
    name: [u8;32],
    head: FileListNode,
}

impl FileListNode {
    pub const fn new(size: usize, data: [u8;BYTE_LENGTH]) -> Self{FileListNode {size, data, next: None }}
    pub fn start_addr(&self) -> usize {
        self as *const Self as usize
    }
    pub fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

pub fn createFile(name: &str) -> File{
    let file_name = str_to_u8(name);
    File::new(file_name)
}

impl File {
    pub const fn new(name: [u8;32]) -> Self {
        Self {
            name,
            head: FileListNode::new(1, [0; BYTE_LENGTH]),
        }
    }

    pub fn write(&mut self, data: &str) {
        let node_count = (data.len() / BYTE_LENGTH) + 1;
        let mut rest_length = BYTE_LENGTH;
        let mut node = &mut self.head;

        for node_counter in 0..node_count {
            /*if node_counter > 0 {
                if node.next.is_none() {
                    node.next = Some(* Box::new(&mut FileListNode::new(1, [0; BYTE_LENGTH])))
                }
                node = &mut node.next.unwrap();
                rest_length = rest_length + BYTE_LENGTH;
            }*/

            if node_counter == node_count-1 {
                rest_length = data.len() % BYTE_LENGTH;
            }

            let str_part = &data[(node_counter*BYTE_LENGTH)..rest_length];
            node.data = str_to_file(str_part);
        }
    }

    pub fn read(&mut self, length: i64) -> Vec<[u8;BYTE_LENGTH]> { // length = bytes
        let mut content_vec: Vec<[u8;BYTE_LENGTH]> = Vec::new();
        let node_count = length as usize;
        let mut selected_node = &self.head;

        for node_counter in 0..node_count {
            if node_counter > 0 {
                /*if selected_node.next.is_some() {
                    selected_node = &selected_node.next.unwrap()
                }
                else {
                    panic!("Data too short!");
                }*/
            }

            content_vec.push(selected_node.data)
        }

        content_vec
    }

    pub fn empty(&mut self) {
        self.head = FileListNode::new(1, [0; BYTE_LENGTH]);
    }
}