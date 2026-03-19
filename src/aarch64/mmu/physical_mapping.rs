use crate::aarch64::mmu::{PhysAddr, GRANULARITY, L2_BLOCK_BITMAP_WORDS, MAX_PAGE_COUNT};
use nova_error::NovaError;

static mut PAGING_BITMAP: [u64; MAX_PAGE_COUNT / 64] = [0; MAX_PAGE_COUNT / 64];

pub fn reserve_page() -> PhysAddr {
    if let Some(address) = find_unallocated_page() {
        let page = address / GRANULARITY;
        let word_index = page / 64;
        unsafe { PAGING_BITMAP[word_index] |= 1 << (page % 64) };
        return address;
    }
    panic!("Out of Memory!");
}

pub fn reserve_page_explicit(physical_address: usize) -> Result<PhysAddr, NovaError> {
    let page = physical_address / GRANULARITY;
    let word_index = page / 64;

    if unsafe { PAGING_BITMAP[word_index] } & (1 << (page % 64)) > 0 {
        return Err(NovaError::Paging);
    }

    unsafe { PAGING_BITMAP[word_index] |= 1 << (page % 64) };
    Ok(physical_address)
}

pub fn reserve_block() -> usize {
    if let Some(start) = find_contiguous_free_bitmap_words(L2_BLOCK_BITMAP_WORDS) {
        for j in 0..L2_BLOCK_BITMAP_WORDS {
            unsafe { PAGING_BITMAP[start + j] = u64::MAX };
        }
        return start * 64 * GRANULARITY;
    }

    panic!("Out of Memory!");
}

pub fn reserve_block_explicit(physical_address: usize) -> Result<(), NovaError> {
    let page = physical_address / GRANULARITY;
    for i in 0..L2_BLOCK_BITMAP_WORDS {
        unsafe {
            if PAGING_BITMAP[(page / 64) + i] != 0 {
                return Err(NovaError::Paging);
            }
        };
    }
    for i in 0..L2_BLOCK_BITMAP_WORDS {
        unsafe {
            PAGING_BITMAP[(page / 64) + i] = u64::MAX;
        };
    }
    Ok(())
}

fn find_unallocated_page() -> Option<usize> {
    for (i, entry) in unsafe { PAGING_BITMAP }.iter().enumerate() {
        if *entry != u64::MAX {
            for offset in 0..64 {
                if entry >> offset & 0b1 == 0 {
                    return Some((i * 64 + offset) * GRANULARITY);
                }
            }
        }
    }
    None
}

fn find_contiguous_free_bitmap_words(required_words: usize) -> Option<usize> {
    let mut run_start = 0;
    let mut run_len = 0;

    for (i, entry) in unsafe { PAGING_BITMAP }.iter().enumerate() {
        if *entry == 0 {
            if run_len == 0 {
                run_start = i;
            }
            run_len += 1;

            if run_len == required_words {
                return Some(run_start);
            }
        } else {
            run_len = 0;
        }
    }

    None
}
