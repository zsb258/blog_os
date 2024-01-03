#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};
use x86_64::VirtAddr;

use blog_os::allocator;
use blog_os::memory;
use blog_os::println;

entry_point!(kernel_main);

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

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello World{}", "!");
    // panic!("Some panic message");

    blog_os::init();

    // init heap
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    {
        // fn stack_overflow() {
        //     stack_overflow(); // for each recursion, the return address is pushed
        // }

        // trigger a stack overflow
        // stack_overflow();
    }

    {
        // trigger a page fault
        // unsafe {
        //     *(0xdeadbeef as *mut u8) = 42;
        // };

        // points to a read-only page
        // let ptr = 0x2052ea as *mut u8;
        // read from a code page
        // unsafe {
        //     let _ = *ptr;
        // }
        // println!("read worked");

        // write to a code page
        // unsafe {
        //     *ptr = 42;
        // }
        // println!("write worked");

        // access level 4 page table
        // use x86_64::registers::control::Cr3;
        //
        // let (level_4_page_table, _) = Cr3::read();
        // println!(
        //     "Level 4 page table at: {:?}",
        //     level_4_page_table.start_address()
        // );
    }

    {
        // use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
        //
        // use x86_64::structures::paging::{Page, Translate};
        //
        // let addresses = [
        //     // the identity-mapped vga buffer page
        //     0xb8000,
        //     // some code page
        //     0x201008,
        //     // some stack page
        //     0x0100_0020_1a10,
        //     // virtual address mapped to physical address 0
        //     boot_info.physical_memory_offset,
        // ];
        //
        // for &address in &addresses {
        //     let virt = VirtAddr::new(address);
        //     let phys = mapper.translate_addr(virt);
        //     println!("{:?} -> {:?}", virt, phys);
        // }
        //
        // map a page to VGA buffer frame
        // let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
        // memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);
        //
        // write the string `New!` to the screen through the new mapping
        // let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
        // unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };
        // // allocate a number on the heap
        // let heap_value = Box::new(41);
        //
        // println!("heap_value at {:p}", heap_value);
        //
        // // create a dynamically sized vector
        // let mut vec = Vec::new();
        // for i in 0..500 {
        //     vec.push(i);
        // }
        // println!("vec at {:p}", vec.as_slice());
        //
        // // create a reference counted vector -> will be freed when count reaches 0
        // let reference_counted = Rc::new(vec![1, 2, 3]);
        // let cloned_reference = reference_counted.clone();
        // println!(
        //     "current reference count is {}",
        //     Rc::strong_count(&cloned_reference)
        // );
        // core::mem::drop(reference_counted);
        // println!(
        //     "reference count is {} now",
        //     Rc::strong_count(&cloned_reference)
        // );
    }

    #[cfg(test)]
    test_main();

    println!("It did not crash!");

    {
        use blog_os::task::executor::Executor;
        use blog_os::task::keyboard;
        use blog_os::task::Task;

        async fn async_number() -> u32 {
            42
        }

        async fn example_task() {
            let number = async_number().await;
            println!("async number: {}", number);
        }

        let mut executor = Executor::new();
        executor.spawn(Task::new(example_task()));
        executor.spawn(Task::new(keyboard::print_keypresses()));
        executor.run() // never returns
    }
}
