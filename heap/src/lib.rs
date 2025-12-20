#![allow(static_mut_refs)]
#![cfg_attr(not(test), no_std)]

use core::{
    alloc::GlobalAlloc,
    mem::size_of,
    prelude::v1::*,
    ptr::{self, null_mut},
    result::Result,
};

use NovaError::NovaError;

extern crate alloc;

#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct HeapHeader {
    next: Option<*mut HeapHeader>,
    before: Option<*mut HeapHeader>,
    size: usize,
    free: bool,
}

const HEAP_HEADER_SIZE: usize = size_of::<HeapHeader>();
const MIN_BLOCK_SIZE: usize = 16;

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

        self.raw_size = heap_end - heap_start + 1;

        unsafe {
            ptr::write(
                self.start_address,
                HeapHeader {
                    next: None,
                    before: None,
                    size: self.raw_size - HEAP_HEADER_SIZE,
                    free: true,
                },
            );
        }
    }

    unsafe fn find_first_fit(&self, size: usize) -> Result<*mut HeapHeader, NovaError> {
        let mut current = self.start_address;
        while !fits(size, current) {
            if let Some(next) = (*self.start_address).next {
                current = next;
            } else {
                return Err(NovaError::HeapFull);
            }
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
        if let Some(next) = next {
            (*next).before = Some(new_address);
        }

        unsafe {
            ptr::write(
                new_address as *mut HeapHeader,
                HeapHeader {
                    next,
                    before: Some(current),
                    size: (*current).size - byte_offset,
                    free: true,
                },
            )
        };
        (*current).next = Some(new_address);
        (*current).free = false;
        (*current).size = size;
    }

    pub fn free(&self, pointer: *mut u8) -> Result<(), NovaError> {
        let mut segment = Self::get_header_ref_from_data_pointer(pointer);
        unsafe {
            // IF prev is free:
            // Delete header, add size to previous and fix pointers.
            // Move Head left
            if let Some(before_head) = (*segment).before {
                if (*before_head).free {
                    (*before_head).size += (*segment).size + HEAP_HEADER_SIZE;
                    delete_header(segment);
                    segment = before_head;
                }
            }

            // IF next is free:
            // Delete next header and merge size, fix pointers
            if let Some(next_head) = (*segment).next {
                if (*next_head).free {
                    (*segment).size += (*next_head).size + HEAP_HEADER_SIZE;
                    delete_header(next_head);
                }
            }

            // Neither: Set free
            (*segment).free = true;
        }

        Ok(())
    }

    const fn get_header_ref_from_data_pointer(pointer: *mut u8) -> *mut HeapHeader {
        unsafe { pointer.sub(HEAP_HEADER_SIZE) as *mut HeapHeader }
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
    let before_opt = (*header).before;
    let next_opt = (*header).next;

    if let Some(before) = before_opt {
        (*before).next = next_opt;
    }

    if let Some(next) = next_opt {
        (*next).before = before_opt;
    }
}

#[cfg(test)]
mod tests;
