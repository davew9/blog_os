use alloc::{vec::Vec, boxed::Box};
use crate::filesystem::{str_to_file};

const BYTE_LENGTH:usize = 1024;

pub struct FileListNode {
    //TODO nicht den Heap als Speicher verwenden, sondern eigenen Bereich
    size: usize,
    data: [u8;BYTE_LENGTH], // 1024 = 1 Byte
    next: Option<Box<FileListNode>>,
}

pub struct File{
    head: FileListNode,
}

impl FileListNode {
    pub const fn new(size: usize, data: [u8;BYTE_LENGTH]) -> Self{
        FileListNode {size, data, next: None }
    }

    pub fn change_data(&mut self, data: [u8;BYTE_LENGTH]) {
        self.data = data;
    }

    pub fn get_me(&mut self) -> &mut FileListNode {
        return self;
    }

    pub fn add_node(&mut self, data: [u8;BYTE_LENGTH]) {
        match &mut self.next {
            None => {
                let new_node = FileListNode{
                    size: BYTE_LENGTH,
                    data,
                    next: None,
                };
                self.next = Some(Box::new(new_node));
            },
            Some(ref mut nextNode) => nextNode.add_node(data),
        }
    }

    pub fn get_next_node(&mut self) -> Option<&mut FileListNode>{
        return match self.next {
            None => {
                None
            },
            Some(ref mut nextNode) => {
                Some(nextNode)
            },
        }
    }
}

impl File {
    pub const fn new() -> Self {
        Self {
            head: FileListNode::new(BYTE_LENGTH, [0; BYTE_LENGTH]),
        }
    }

    pub fn write(&mut self, data: &str) {
        //TODO Node is filled up zeroes, distorts content
        let node_count = (data.len() / BYTE_LENGTH) + 1;
        let mut rest_length = 0;
        let mut node = &mut self.head;

        for node_counter in 0..node_count {
            if node_counter == node_count-1 {
                rest_length = rest_length + data.len() % BYTE_LENGTH;
            }
            else {
                rest_length = rest_length + BYTE_LENGTH;
            }

            let str_part = &data[(node_counter*BYTE_LENGTH)..rest_length];

            if node_counter > 0 {
                if node.next.is_none() {
                    node.add_node(str_to_file(str_part));
                    println!("NODE ADDED!")
                }
                else {
                    node = node.get_next_node().unwrap();
                    node.change_data(str_to_file(str_part));
                }
            }
            else {
                node.change_data(str_to_file(str_part));
            }
        }
    }

    //TODO Write append function to append new data to the end of the file

    pub fn read(&mut self, length: i64) -> Vec<[u8;BYTE_LENGTH]> { // length = bytes
        let mut content_vec: Vec<[u8;BYTE_LENGTH]> = Vec::new();
        let node_count = length as usize;
        let mut selected_node = &mut self.head;

        for node_counter in 0..node_count {
            if node_counter > 0 {
                if selected_node.get_next_node().is_some() {
                    selected_node = selected_node.get_next_node().unwrap();
                    content_vec.push(selected_node.data)
                }
                else {
                    return content_vec;
                }
            }
            else {
                content_vec.push(selected_node.data)
            }
        }

        return content_vec;
    }

    pub fn empty(&mut self) {
        self.head = FileListNode::new(1, [0; BYTE_LENGTH]);
    }
}