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
pub struct HeapHeader {
    pub next: *mut HeapHeader,
    before: *mut HeapHeader,
    pub size: usize,
    free: bool,
}

const HEAP_HEADER_SIZE: usize = size_of::<HeapHeader>();
const MIN_BLOCK_SIZE: usize = 16;

#[derive(Default)]
pub struct Novalloc;

unsafe impl GlobalAlloc for Novalloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        malloc(layout.size()).unwrap()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        free(ptr).unwrap();
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: Novalloc = Novalloc;

pub fn init_heap() {
    unsafe {
        let heap_end = &raw const __heap_end as usize;
        let heap_start = &raw const __heap_start as usize;

        ptr::write(
            &raw const __heap_start as *mut HeapHeader,
            HeapHeader {
                next: null_mut(),
                before: null_mut(),
                size: heap_end - heap_start - HEAP_HEADER_SIZE,
                free: true,
            },
        );
    }
}

pub fn malloc(mut size: usize) -> Result<*mut u8, NovaError> {
    let mut head = &raw const __heap_start as *mut HeapHeader;

    if size == 0 {
        return Err(NovaError::EmptyHeapNotAllowed);
    }

    if size < MIN_BLOCK_SIZE {
        size = MIN_BLOCK_SIZE;
    }

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

        // Return entire block WITHOUT generating a new header
        // if the current block doesn't have enough space to hold: requested size + HEAP_HEADER_SIZE + MIN_BLOCK_SIZE
        if (*head).size < size + HEAP_HEADER_SIZE + MIN_BLOCK_SIZE {
            (*head).free = false;
            return Ok(head.byte_add(HEAP_HEADER_SIZE) as *mut u8);
        }

        let byte_offset = HEAP_HEADER_SIZE + size;
        let new_address = head.byte_add(byte_offset);

        // Handle case where fragmenting center free space
        let next = (*head).next;
        if !(*head).next.is_null() {
            (*next).before = new_address;
        }

        ptr::write(
            new_address as *mut HeapHeader,
            HeapHeader {
                next,
                before: head,
                size: (*head).size - size - HEAP_HEADER_SIZE,
                free: true,
            },
        );
        (*head).next = new_address;
        (*head).free = false;
        (*head).size = size;

        let data_start_address = head.byte_add(HEAP_HEADER_SIZE);

        Ok(data_start_address as *mut u8)
    }
}

pub fn free(pointer: *mut u8) -> Result<(), NovaError> {
    let mut head = unsafe { pointer.sub(HEAP_HEADER_SIZE) as *mut HeapHeader };
    unsafe {
        // IF prev is free:
        // Delete header, add size to previous and fix pointers.
        // Move Head left
        if !(*head).before.is_null() && (*(*head).before).free {
            let before_head = (*head).before;
            (*before_head).size += (*head).size + HEAP_HEADER_SIZE;
            delete_header(head);
            head = before_head;
        }
        // IF next is free:
        // Delete next header and merge size, fix pointers
        if !(*head).next.is_null() && (*(*head).next).free {
            let next_head = (*head).next;
            (*head).size += (*next_head).size + HEAP_HEADER_SIZE;
            delete_header(next_head);
        }
        // Neither: Set free
        (*head).free = true;
    }

    Ok(())
}

unsafe fn delete_header(header: *mut HeapHeader) {
    let before = (*header).before;
    let next = (*header).next;

    if !before.is_null() {
        (*before).next = next;
    }

    if !next.is_null() {
        (*next).before = before;
    }
}

pub fn traverse_heap_tree() {
    let mut pointer_address = &raw const __heap_start as *const HeapHeader;
    loop {
        let head = unsafe { read_volatile(pointer_address) };
        println!("Header {:#x}", pointer_address as u32);
        println!("free: {}", head.free);
        println!("size: {}", head.size);
        println!("hasNext: {}", !head.next.is_null());
        println!();
        if !head.next.is_null() {
            pointer_address = head.next;
        } else {
            println!("---------------");
            return;
        }
    }
}
