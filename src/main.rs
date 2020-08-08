
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use blog_os::println;
use blog_os::task::{executor::Executor, keyboard, Task};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use blog_os::vga_buffer::print_bytes;
use blog_os::filesystem;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use blog_os::allocator;
    use blog_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    println!("Hello World{}", "!");
    blog_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    #[cfg(test)]
        test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(example_task2()));
    executor.spawn(Task::new(example_task3()));
    executor.spawn(Task::new(keyboard::print_keypresses()));

    executor.spawn(Task::new(example_task_file_type()));
    executor.run();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    blog_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    println!("creating the file test1 and test 2 in task1");
    filesystem::create("test1");
    // filesystem::create("test2");
    let file1 = filesystem::open("test1").unwrap();
    //let file2 = filesystem::open("test2");
    println!("writing content in test1 and test2");
    filesystem::write(file1, 0, "Content of File 1: Should be also visible in task2");
    //filesystem::write(file2, 0, "Content of File 2: Should be also visible in task2");
    let content1 = filesystem::read(file1,0);
    //let content2 = filesystem::read(file4,0);
    println!("content of file 1 accessed in task1:");
    print_bytes(&content1.unwrap());
    filesystem::close(file1);
    //file2 stays open
}

async fn example_task2() {
    let file3 = filesystem::open("test1").unwrap();
    //let file4 = filesystem::open("test2");
    let content1 = filesystem::read(file3,0);
    //let content2 = filesystem::read(file4,0);
    println!("content of file 1 accessed in task2:");
    print_bytes(&content1.unwrap());
    // println!("content of file 2 accessed in task2:");
    // print_bytes(&content2.unwrap());
    filesystem::create_dir("directory2");
    filesystem::create("directory1/directory1/test2");
    filesystem::create("directory1/directory2/test2");
    let file5 = filesystem::open("directory1/directory2/test2").unwrap();
    filesystem::write(file5, 0, "this is pretty nested");
    filesystem::close(file5);
    let file6 = filesystem::open("directory1/directory2/test2").unwrap();
    let content6 = filesystem::read(file6, 0);
    print_bytes(&content6.unwrap());
}

async fn example_task3() {
    filesystem::delete("directory1/directory2/test2");
    let file7 = filesystem::open("directory1/directory2/test2").unwrap();
    let content7 = filesystem::read(file7, 0);
    print_bytes(&content7.unwrap());
    filesystem::delete("directory1/directory1");
    filesystem::delete("directory1/directory1/test2");
    filesystem::delete("directory1/directory1");
    filesystem::create_dir("directory1/directory1/directory5");
    filesystem::open("directory1");
    filesystem::open("directory1/directory1/test2");
    filesystem::create("1/2/3/4/5/6/file");
    let file8 = filesystem::open("1/2/3/4/5/6/file").unwrap();
    filesystem::read(file8,0);
    filesystem::open("1/2/3/4/5/6");
    filesystem::delete("1/2/3/4/5/6/file");
    filesystem::close(file8);
    filesystem::delete("1/2/3/4/5/6/file");
    filesystem::delete("1/2/3/4/5");
    filesystem::delete("1/2/3/4/5/6");
    filesystem::delete("1/2/3/4/5");
}

async fn example_task_file_type() {
    use filesystem::file::File;

    // INHALT 1 - SCHREIBEN UND LESEN
    let mut test_file = File::new();
    test_file.write("aaaaaaaaaaaaaaaaaaaaa");

    let content = test_file.read(1);

    println!("Content of TEST-FILE:");
    for letter in content {
        print_bytes(&letter)
    }

    // INHALT 2 - SCHREIBEN, LEEREN UND LESEN -> KEINE AUSGABE
    test_file.write("test");
    test_file.empty();
    let content = test_file.read(1);

    println!("Content of TEST-FILE:");
    for letter in content {
        print_bytes(&letter)
    }

    // INHALT 3 - SCHREIBEN ÜBER MEHERERE NODS, LESEN
    test_file.write("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbaaaaaaaaabbbbbbbbaaaaaaaaabbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbaaaaaaaaabbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbaaaaaaaaabbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbaaaaaaaaabbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbaaaaaaaaabbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbaaaaaaaaabbbbbbbbbbbbbbbbbbbbbbbZZZZbbbbbbbbbbbbbbbbbaaaaaaaaabbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbaaaaaaaaabbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbaaaaaaaaabbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbaaaaaaaaabbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbaaaaaaaaabbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbaaaaaaaaabbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbaabbbbbbbbaaaaaaaaabbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbaaaaaaaaabbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbaaaaaaaaabbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbaaaaaaaaabbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbaaaaaaaaabbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbaaaaaabbbbbbbbbbbbbbbbbbbbbbbbbbbbaaaaaaaaabbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbaaaaaaaaabbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbaaaaaaaaabbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbaaaaaaaaabbbbbbbbbbbbbbbbbbbbbbbbbbZZZbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbZZZ");
    let content = test_file.read(3);

    println!("Content of TEST-FILE:");
    for letter in content {
        print_bytes(&letter)
    }
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}