use alloc::{boxed::Box, vec::Vec};
use blog_os::allocator;

pub struct FileListNode {
    size: usize,
    data: Box<char>,
    next: Option<&'static mut FileListNode>
}

pub struct File{
    name: u8,
    file: Option<&'static mut FileListNode>,
    allocator: BootInfoFrameAllocator,
}

impl FileListNode {
    const fn new(size: usize) -> Self{FileListNode {size, data: None, next: None }}
    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }
    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

impl File {
    const fn new(name: u8, allocator: BootInfoFrameAllocator) -> Self{File{name, file: None, allocator}}

    fn write(data: char) {
        if {file == None} {
            file = Box::new(data)
        }
        else {
            file.data = data;
        }
    }

    fn read(length: i64) {
        let content_vec = Vec::new();
        let file_node = file;

        for i in 0..length {
            content_vec.push(file_node.data);
            file_node = file.next;
        }

        content_vec
    }

    fn delete(&self) {self.file = None}
}