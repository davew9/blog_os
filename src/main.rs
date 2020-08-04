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
    executor.spawn(Task::new(keyboard::print_keypresses()));
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
    filesystem::create("test2");
    let file1 = filesystem::open("test1");
    let file2 = filesystem::open("test2");
    println!("writing content in test1 and test2");
    filesystem::write(file1, 0, "Content of File 1: Should be also visible in task2");
    filesystem::write(file2, 0, "Content of File 2: Should be also visible in task2");
    filesystem::close(file1);
    //file2 stays open
}

async fn example_task2() {
    let file3 = filesystem::open("test1");
    let file4 = filesystem::open("test2");
    let content1 = filesystem::read(file3,0);
    let content2 = filesystem::read(file4,0);
    println!("content of file 1 accessed in task2:");
    print_bytes(&content1.unwrap());
    println!("content of file 2 accessed in task2:");
    print_bytes(&content2.unwrap());

}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
