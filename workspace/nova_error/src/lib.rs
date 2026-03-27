#![no_std]

use core::fmt::Debug;
use core::prelude::rust_2024::derive;

#[derive(Debug)]
pub enum NovaError {
    General(&'static str),
    Mailbox,
    HeapFull,
    EmptyHeapSegmentNotAllowed,
    Misalignment,
    InvalidGranularity,
    Paging(&'static str),
    OutOfMeomory,
}
