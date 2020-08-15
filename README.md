# Blog OS Filesystem Extension

This repository contains the source code for a Filesystem Extension for the Rust OS written by Philipp Oppermann.

**Check out the [master branch](https://github.com/phil-opp/blog_os) or the associated [blog](https://os.phil-opp.com) for more  information about the project by Phillip Oppermann.**

# Summary
This extension contains a simple in-memory filesystem, which offers methods to create, edit and delete files. Furthermore a directory system is implemented. Particular attention is given regarding the deterministic aspects of the system in a RTOS context. Some aspects of determism might be dealt only theoretically, though. 
For simplicity the Heap-Memory is used to save data. The structure of a file itself is composed of a linked list.
A very limited CLI is provided for demonstrating and testing of the filesystem.


# Functionality
### Offered Methods
**open():** Returns a task specific handle to the file specified by the path String. The file cannot be deleted while it's open in any task, but i might be read or edited.    
**close():** Closes the file specified by a task specific handle. Therefore the handle becomes invalid.     
**read()**: Returns the content of the file specified by a task specific handle. During the read operation the content of the file cannot be changed by any other process, but it might be edited afterwards.    
**write()**: Changes the content of the file specified by a task specific handle. During the write operation the content of the file cannot be changed or read by any other process, but it might be edited afterwards. Offers the possibilty to append data.    
**delete()**: Deletes a file or directory specified by a path String. Files cannnot be deleted while they are open in any task. Directories cannot be deleted if they contain other directories or files.    
**create_dir()**: Creates a directory specified by a path String. If the path contains directories which don't exist, these directories are also created.    
**create()**: Creates a file specified by a path String. If the path contains directories which don't exist, these directories are also created.    


### Command Line Interface
- "/" might be bound to the # key
- Backspace discards the current input
#### Commands
**mkdir \<path\>**: Creates directory specified by path and all not existing parent directories    
**mkfile \<path\>**: Creates file specified by path and all not existing parent directories     
**rm \<path\>**: Deletes file or directorie specified by path    
**show \<path\> (\<offset in kb\>)**: Shows content of file specified by path and offset   
**edit \<path\> \<txt\>**: Replaces content of file specified by path with txt    
**apd \<path\> \<txt\>**: Appends txt to file specified by path     
**cd \<path\>**: Changes working directory according to path    

# Implementation Details
## Concurrent Access
All system tables which might be concurrently accessed are guarded by a semaphore based Read-Write-Lock. The Read-Lock still allows for atomic operations. Therefore the read() and write() operations can be executed fully concurrently if they don't concern the same file. The open(), close(), rm(), and create() operations need an exclusive lock for certain structures and are also generally more time consuming. In a RTOS context its recommended to execute those operations as rarely as possible. For example only at the start and at the shutdown of a system.

The files themselves are also protected by a Read-Write-Lock Mechanism. A lock will only be held as long as the file is actively accessed. Therefore the possibility to come upon a locked file is drastically reduced. On the other hand, a task has no chance to prevent or detect a change in data between two file system calls.


## File Structure
Files are saved as a Linked List. This structure gives us the advantage, that you can store the file on distributed memory parts. You are not dependent on having coherent blocks in the memory. Files can grow and shrink easily. In this prototype one node of the linked list consists a 1 kb data storage as an array of u8 and a pointer to the next node. If you write information to a file the file type divides the data into multiple parts and creates the required amount of nodes. Another possibility is appending information to an existing file. The append function jumps to the last existing node and checks if there is free space. The free space will be filled up with your data. Otherwise more nodes will be created. 

To read from a file you can use the read function. It collects the wanted data by iterating over the nodes and returns the data parts e.g. as a string.  If you want to delete the stored information the nodes get deleted. 

Currently the used memory is placed in the heap. Because of the file concept you can easily move the file type to another storage unit.  

## Limitations
- Paths must not be longer than 32 characters
- Not all edge cases might be covered

## Building

This project requires a nightly version of Rust because it uses some unstable features. At least nightly _2020-07-15_ is required for building. You might need to run `rustup update nightly --force` to update to the latest nightly even if some components such as `rustfmt` are missing it.

You can build the project by running:

```
cargo build
```

To create a bootable disk image from the compiled kernel, you need to install the [`bootimage`] tool:

[`bootimage`]: https://github.com/rust-osdev/bootimage

```
cargo install bootimage
```

After installing, you can create the bootable disk image by running:

```
cargo bootimage
```

This creates a bootable disk image in the `target/x86_64-blog_os/debug` directory.

Please file an issue if you have any problems.

## Running

You can run the disk image in [QEMU] through:

[QEMU]: https://www.qemu.org/

```
cargo run
```

[QEMU] and the [`bootimage`] tool need to be installed for this.

You can also write the image to an USB stick for booting it on a real machine. On Linux, the command for this is:

```
dd if=target/x86_64-blog_os/debug/bootimage-blog_os.bin of=/dev/sdX && sync
```

Where `sdX` is the device name of your USB stick. **Be careful** to choose the correct device name, because everything on that device is overwritten.

## Testing

To run the unit and integration tests, execute `cargo xtest`.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Note that this only applies to this git branch, other branches might be licensed differently.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
