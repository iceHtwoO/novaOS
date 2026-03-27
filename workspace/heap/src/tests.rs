use super::*;
use rand::{self, random_range};
extern crate std;

static HEAP_SIZE: usize = 1024;

#[test]
fn test_heap_allocation() {
    let heap_vector = Box::new([0u8; HEAP_SIZE]);
    let mut heap = Heap::empty();
    heap.init(
        &heap_vector[0] as *const u8 as usize,
        &heap_vector[HEAP_SIZE - 1] as *const u8 as usize,
    );

    let root_header = heap.start_address;

    let malloc_size = random_range(0..(HEAP_SIZE - HEAP_HEADER_SIZE));
    let malloc = heap.malloc(malloc_size).unwrap();
    let malloc_header = Heap::get_header_ref_from_data_pointer(malloc);

    assert_eq!(root_header, malloc_header);

    unsafe {
        let actual_alloc_size = (*malloc_header).size;
        let actual_raw_size = actual_alloc_size + HEAP_HEADER_SIZE;
        // Verify sizing
        assert!(actual_alloc_size >= malloc_size);
        assert_eq!(actual_alloc_size % MIN_BLOCK_SIZE, 0);

        // Verify section is occupied
        assert!(!(*malloc_header).free);

        // Verify next header has been created
        let next = (*malloc_header).next.unwrap();

        assert_eq!(malloc_header.byte_add(actual_raw_size), next);
        assert!((*next).free);
        assert_eq!((*malloc_header).next.unwrap(), next);
        assert_eq!((*next).before.unwrap(), malloc_header);
        assert_eq!((*next).size, HEAP_SIZE - actual_raw_size - HEAP_HEADER_SIZE)
    }
}

#[test]
fn test_full_heap() {
    let heap_vector = Box::new([0u8; HEAP_SIZE]);
    let mut heap = Heap::empty();
    heap.init(
        &heap_vector[0] as *const u8 as usize,
        &heap_vector[HEAP_SIZE - 1] as *const u8 as usize,
    );

    let malloc_size = HEAP_SIZE - HEAP_HEADER_SIZE;
    let malloc = heap.malloc(malloc_size).unwrap();
    let malloc_header = Heap::get_header_ref_from_data_pointer(malloc);
    unsafe {
        assert!(!(*malloc_header).free);
        assert!((*malloc_header).next.is_none());
    }

    let malloc2 = heap.malloc(MIN_BLOCK_SIZE);
    assert!(malloc2.is_err());
}

#[test]
fn test_freeing_root() {
    let heap_vector = Box::new([0u8; HEAP_SIZE]);
    let mut heap = Heap::empty();
    heap.init(
        &heap_vector[0] as *const u8 as usize,
        &heap_vector[HEAP_SIZE - 1] as *const u8 as usize,
    );

    let root_header = heap.start_address;
    let root_header_start_size = unsafe { (*root_header).size };

    let malloc_size = random_range(0..((HEAP_SIZE - HEAP_HEADER_SIZE) / 2));
    let malloc = heap.malloc(malloc_size).unwrap();
    let malloc_header = Heap::get_header_ref_from_data_pointer(malloc);
    unsafe {
        assert!(!(*malloc_header).free);
        assert!((*malloc_header).size >= malloc_size);
        assert!((*root_header).next.is_some());

        assert!(heap.free(malloc).is_ok());

        assert_eq!((*root_header).size, root_header_start_size);
        assert!((*root_header).next.is_none());
    }
}

#[test]
fn test_merging_free_sections() {
    let heap_vector = Box::new([0u8; HEAP_SIZE]);
    let mut heap = Heap::empty();
    heap.init(
        &heap_vector[0] as *const u8 as usize,
        &heap_vector[HEAP_SIZE - 1] as *const u8 as usize,
    );

    let root_header = heap.start_address;
    let _root_header_start_size = unsafe { (*root_header).size };

    let malloc1 = heap.malloc(MIN_BLOCK_SIZE).unwrap();
    let malloc_header_before = unsafe { *Heap::get_header_ref_from_data_pointer(malloc1) };
    let malloc2 = heap.malloc(MIN_BLOCK_SIZE).unwrap();
    let _ = heap.malloc(MIN_BLOCK_SIZE).unwrap();

    unsafe {
        assert!(heap.free(malloc1).is_ok());

        let malloc_header_free = *Heap::get_header_ref_from_data_pointer(malloc1);
        assert_ne!(malloc_header_before.free, malloc_header_free.free);
        assert_eq!(malloc_header_before.size, malloc_header_free.size);

        assert!(heap.free(malloc2).is_ok());
        let malloc_header_merge = *Heap::get_header_ref_from_data_pointer(malloc1);

        assert!(malloc_header_merge.free);
        assert_eq!(
            malloc_header_merge.size,
            malloc_header_free.size + MIN_BLOCK_SIZE + HEAP_HEADER_SIZE
        );
    }
}

#[test]
fn test_first_fit() {
    let heap_vector = Box::new([0u8; HEAP_SIZE]);
    let mut heap = Heap::empty();
    heap.init(
        &heap_vector[0] as *const u8 as usize,
        &heap_vector[HEAP_SIZE - 1] as *const u8 as usize,
    );

    let root_header = heap.start_address;
    let _root_header_start_size = unsafe { (*root_header).size };

    let malloc1 = heap.malloc(MIN_BLOCK_SIZE).unwrap();
    let _malloc2 = heap.malloc(MIN_BLOCK_SIZE).unwrap();
    let malloc3 = heap.malloc(MIN_BLOCK_SIZE * 3).unwrap();
    let malloc4 = heap.malloc(MIN_BLOCK_SIZE).unwrap();

    assert!(heap.free(malloc1).is_ok());
    assert!(heap.free(malloc3).is_ok());
    let malloc5 = heap.malloc(MIN_BLOCK_SIZE * 2).unwrap();
    let malloc1_header = unsafe { *Heap::get_header_ref_from_data_pointer(malloc1) };

    // First free block stays empty
    assert!(malloc1_header.free);

    // New allocation takes the first fit aka. malloc3
    assert_eq!(malloc5, malloc3);

    // If no free slot could be found, append to the end
    let malloc6 = heap.malloc(MIN_BLOCK_SIZE * 2).unwrap();
    assert!(malloc6 > malloc4);

    // Malloc7 takes slot of Malloc1
    let malloc7 = heap.malloc(MIN_BLOCK_SIZE).unwrap();
    assert_eq!(malloc1, malloc7);
}
