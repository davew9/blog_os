use alloc::{vec::Vec, boxed::Box};
use crate::filesystem::{str_to_file};

const BYTE_LENGTH:usize = 1024;

// Linked List Node contains part of the file data
#[allow(dead_code)]
pub struct FileListNode {
    //TODO nicht den Heap als Speicher verwenden, sondern eigenen Bereich
    size: usize,
    data: [u8;BYTE_LENGTH], // 1024 = 1 Byte
    next: Option<Box<FileListNode>>,
}

// Start node of the file
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

    pub fn append_data(&mut self, data: [u8; BYTE_LENGTH], index: usize, length: usize) {
        let mut counter = 0;
        for entry in self.data.iter_mut() {
            if counter >= index {
                *entry = data[counter - index];
            }
            if counter > index+length {
                break;
            }
            counter = counter + 1;
        }
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
            Some(ref mut next_node) => next_node.add_node(data),
        }
    }

    pub fn get_next_node(&mut self) -> Option<&mut FileListNode>{
        return match self.next {
            None => {
                None
            },
            Some(ref mut next_node) => {
                Some(next_node)
            },
        }
    }

    pub fn find_zero_in_data_array(&mut self) -> usize {
        let mut index = BYTE_LENGTH + 1;
        let mut counter = 0;

        for entry in self.data.iter() {
            if *entry == 0 {
                index = counter;
                break;
            }
            counter = counter + 1;
        }

        return index;
    }
}

impl File {
    pub const fn new() -> Self {
        Self {
            head: FileListNode::new(BYTE_LENGTH, [0; BYTE_LENGTH]),
        }
    }

    pub fn write(&mut self, data: &str) {
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

    pub fn append(&mut self, data: &str) {
        // jump to the last node
        let mut current_node = &mut self.head;

        while current_node.next.is_some() {
            current_node = current_node.get_next_node().unwrap();
        }

        // find the last content
        let mut rest_length = data.len();
        let mut no_empty_space_found = false;
        let cn_index = current_node.find_zero_in_data_array();
        if cn_index > BYTE_LENGTH-1 {
            no_empty_space_found = true;
        }

        if no_empty_space_found == false {
            if rest_length > BYTE_LENGTH {
                rest_length = BYTE_LENGTH - cn_index;
            }
            // fill up the last node with data
            let data_part = &data[0..rest_length];
            current_node.append_data(str_to_file(&data_part), cn_index, rest_length);

            rest_length = data.len() - rest_length;
        }

        // write rest data to file
        if rest_length > 0 {
            let new_start_index = data.len() - rest_length;
            rest_length = data.len() - new_start_index;
            let data_part = &data[new_start_index..rest_length];

            let node_count = data_part.len() / BYTE_LENGTH + 1;
            rest_length = 0;

            for node_counter in 0..node_count {
                if node_counter == node_count-1 {
                    rest_length = rest_length + data_part.len() % BYTE_LENGTH;
                }
                else {
                    rest_length = rest_length + BYTE_LENGTH;
                }

                let str_part = &data_part[(node_counter*BYTE_LENGTH)..rest_length];

                if current_node.next.is_none() {
                    current_node.add_node(str_to_file(str_part));
                }
                else {
                    current_node = current_node.get_next_node().unwrap();
                    current_node.change_data(str_to_file(str_part));
                }
            }
        }
    }

    pub fn read(&mut self, range_start: usize, range_end:usize) -> Vec<[u8;BYTE_LENGTH]> { // length = bytes
        let mut content_vec: Vec<[u8;BYTE_LENGTH]> = Vec::new();
        //let node_count = length as usize;
        let mut selected_node = &mut self.head;

        for node_counter in 0..range_end {
            // Every Node not first or last
            if node_counter > 0 {
                if selected_node.get_next_node().is_some() {
                    selected_node = selected_node.get_next_node().unwrap();
                    if (range_start..range_end).contains(&node_counter) {
                        content_vec.push(selected_node.data);
                    }
                }
                // Last Node
                else {
                    return content_vec;
                }
            }
            // First Node
            else {
                if (range_start..range_end).contains(&node_counter) {
                    content_vec.push(selected_node.data);
                }
            }
        }
        return content_vec;
    }

    pub fn empty(&mut self) {
        self.head = FileListNode::new(1, [0; BYTE_LENGTH]);
    }
}