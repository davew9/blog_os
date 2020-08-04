use alloc::{boxed::Box, vec::Vec};
use crate::memory::BootInfoFrameAllocator;

pub struct FileListNode {
    size: usize,
    data: Option<&'static mut Box<char>>,
    next: Option<&'static mut FileListNode>
}

pub struct File{
    name: u8,
    file: Option<&'static mut FileListNode>,
}

impl FileListNode {
    pub const fn new(size: usize, data: Box<char>) -> Self{FileListNode {size, data: Some(char), next: None }}
    pub fn start_addr(&self) -> usize {
        self as *const Self as usize
    }
    pub fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

impl File {
    pub const fn new(name: u8) -> Self{File{name, file: Some(&mut FileListNode::new(1, Box::new('c')))}}

    pub fn write(&mut self, data: char) {
        let node = &mut self.file;

        if node.data == None {
            node.data = Some(&mut Box::new(data));
        }
    }

    pub fn read(&self, length: i64) -> Vec<char> {
        let mut content_vec: Vec<char> = Vec::new();

        for i in 0..length {
            content_vec.push(*self.file.data);
        }

        content_vec
    }

    pub fn delete(&mut self) {self.file = None}
}