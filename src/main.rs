#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use blog_os::println;

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

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    // panic!("Some panic message");

    blog_os::init();

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
        use x86_64::registers::control::Cr3;

        let (level_4_page_table, _) = Cr3::read();
        println!(
            "Level 4 page table at: {:?}",
            level_4_page_table.start_address()
        );
    }

    #[cfg(test)]
    test_main();

    println!("It did not crash!");

    blog_os::hlt_loop();
}
