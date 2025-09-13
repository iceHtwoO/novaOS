use core::{
    alloc::GlobalAlloc,
    ptr::{self, null, null_mut, read_volatile, write_volatile},
};

use crate::NovaError;
extern crate alloc;

extern "C" {
    static mut __heap_start: u8;
    static mut __heap_end: u8;
}

#[repr(C)]
struct Header {
    next: *mut Header,
    before: *mut Header,
    size: usize,
    free: bool,
}

#[derive(Default)]
pub struct Novalloc;

unsafe impl GlobalAlloc for Novalloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        malloc(layout.size()).unwrap()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        todo!()
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: Novalloc = Novalloc;

pub fn init_malloc() {
    unsafe {
        let heap_end = &raw const __heap_end as usize;
        let heap_start = &raw const __heap_start as usize;
        let s = size_of::<Header>();
        ptr::write(
            &raw const __heap_start as *mut Header,
            Header {
                next: null_mut(),
                before: null_mut(),
                size: heap_end - heap_start - size_of::<Header>(),
                free: true,
            },
        );
    }
}

pub fn malloc(mut size: usize) -> Result<*mut u8, NovaError> {
    let mut head = &raw const __heap_start as *mut Header;

    // Align size to the next 16 bytes
    size += (16 - (size % 16)) % 16;

    unsafe {
        // Find First-Fit memory segment
        while !(*head).free || size > (*head).size {
            if (*head).next.is_null() {
                return Err(NovaError::HeapFull);
            }
            head = (*head).next;
        }

        let byte_offset = size_of::<Header>() + size;
        let new_address = head.byte_add(byte_offset);

        // Handle case where free data block is in the center
        let mut next = null_mut();
        if !(*head).next.is_null() {
            next = (*head).next;
            (*next).before = new_address;
        }

        ptr::write(
            new_address as *mut Header,
            Header {
                next,
                before: head,
                size: (*head).size - size - size_of::<Header>(),
                free: true,
            },
        );
        (*head).next = new_address;
        (*head).free = false;
        (*head).size = size;

        let data_start_address = new_address.byte_add(size_of::<Header>());

        Ok(data_start_address as *mut u8)
    }
}

pub fn traverse_heap_tree() {
    let mut pointer_address = &raw const __heap_start as *const Header;
    loop {
        let head = unsafe { read_volatile(pointer_address) };
        println!("Header {}", pointer_address as u32);
        println!("free: {}", head.free);
        println!("size: {}", head.size);
        println!("hasNext: {}", !head.next.is_null());
        if !head.next.is_null() {
            pointer_address = head.next;
        } else {
            println!("---------------");
            return;
        }
    }
}
