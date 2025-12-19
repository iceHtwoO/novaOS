#![allow(static_mut_refs)]
#![cfg_attr(not(test), no_std)]

use core::{
    alloc::GlobalAlloc,
    default::Default,
    mem::size_of,
    prelude::v1::*,
    ptr::{self, null_mut, read_volatile},
    result::Result,
};

use NovaError::NovaError;

#[cfg(not(target_os = "none"))]
extern crate std;

extern crate alloc;

#[repr(C, align(16))]
pub struct HeapHeader {
    pub next: *mut HeapHeader,
    before: *mut HeapHeader,
    pub size: usize,
    free: bool,
}

const HEAP_HEADER_SIZE: usize = size_of::<HeapHeader>();
const MIN_BLOCK_SIZE: usize = 16;

// TODO: This implementation has to be reevaluated when implementing multiprocessing
// Spinlock could be a solution but has its issues:
// https://matklad.github.io/2020/01/02/spinlocks-considered-harmful.html

pub struct Heap {
    pub start_address: *mut HeapHeader,
    pub end_address: *mut HeapHeader,
    pub raw_size: usize,
}
impl Heap {
    pub const fn empty() -> Self {
        Self {
            start_address: null_mut() as *mut HeapHeader,
            end_address: null_mut() as *mut HeapHeader,
            raw_size: 0,
        }
    }

    pub fn init(&mut self, heap_start: usize, heap_end: usize) {
        self.start_address = heap_start as *mut HeapHeader;
        self.end_address = heap_end as *mut HeapHeader;

        self.raw_size = heap_end - heap_start;

        unsafe {
            ptr::write(
                self.start_address,
                HeapHeader {
                    next: null_mut(),
                    before: null_mut(),
                    size: self.raw_size - HEAP_HEADER_SIZE,
                    free: true,
                },
            );
        }
    }

    unsafe fn find_first_fit(&self, size: usize) -> Result<*mut HeapHeader, NovaError> {
        let mut current = self.start_address;
        while !fits(size, current) {
            if (*self.start_address).next.is_null() {
                return Err(NovaError::HeapFull);
            }
            current = (*current).next;
        }
        Ok(current)
    }

    pub fn malloc(&self, mut size: usize) -> Result<*mut u8, NovaError> {
        if size == 0 {
            return Err(NovaError::EmptyHeapSegmentNotAllowed);
        }

        if size < MIN_BLOCK_SIZE {
            size = MIN_BLOCK_SIZE;
        }

        // Align size to the next 16 bytes
        size += (16 - (size % 16)) % 16;

        unsafe {
            // Find First-Fit memory segment
            let current = self.find_first_fit(size)?;

            // Return entire block WITHOUT generating a new header
            // if the current block doesn't have enough space to hold: requested size + HEAP_HEADER_SIZE + MIN_BLOCK_SIZE
            if (*current).size < size + HEAP_HEADER_SIZE + MIN_BLOCK_SIZE {
                (*current).free = false;
                return Ok(current.byte_add(HEAP_HEADER_SIZE) as *mut u8);
            }

            Self::fragment_segment(current, size);

            let data_start_address = current.byte_add(HEAP_HEADER_SIZE);

            Ok(data_start_address as *mut u8)
        }
    }

    unsafe fn fragment_segment(current: *mut HeapHeader, size: usize) {
        let byte_offset = HEAP_HEADER_SIZE + size;
        let new_address = unsafe { current.byte_add(byte_offset) };

        // Handle case where fragmenting center free space
        let next = (*current).next;
        if !(*current).next.is_null() {
            (*next).before = new_address;
        }

        unsafe {
            ptr::write(
                new_address as *mut HeapHeader,
                HeapHeader {
                    next,
                    before: current,
                    size: (*current).size - size - HEAP_HEADER_SIZE,
                    free: true,
                },
            )
        };
        (*current).next = new_address;
        (*current).free = false;
        (*current).size = size;
    }

    pub fn free(&self, pointer: *mut u8) -> Result<(), NovaError> {
        let mut segment = unsafe { pointer.sub(HEAP_HEADER_SIZE) as *mut HeapHeader };
        unsafe {
            // IF prev is free:
            // Delete header, add size to previous and fix pointers.
            // Move Head left
            if !(*segment).before.is_null() && (*(*segment).before).free {
                let before_head = (*segment).before;
                (*before_head).size += (*segment).size + HEAP_HEADER_SIZE;
                delete_header(segment);
                segment = before_head;
            }
            // IF next is free:
            // Delete next header and merge size, fix pointers
            if !(*segment).next.is_null() && (*(*segment).next).free {
                let next_head = (*segment).next;
                (*segment).size += (*next_head).size + HEAP_HEADER_SIZE;
                delete_header(next_head);
            }

            // Neither: Set free
            (*segment).free = true;
        }

        Ok(())
    }
}

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.malloc(layout.size()).unwrap()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _: core::alloc::Layout) {
        self.free(ptr).unwrap();
    }
}

unsafe impl Sync for Heap {}

unsafe fn fits(size: usize, header: *mut HeapHeader) -> bool {
    (*header).free && size <= (*header).size
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

#[cfg(test)]
mod tests {}
