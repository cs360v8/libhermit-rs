// Copyright (c) 2017 Colin Finck, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::arch::riscv64::mm::paging::{BasePageSize, PageSize};
use crate::mm;
use crate::mm::freelist::{FreeList, FreeListEntry};

static mut KERNEL_FREE_LIST: FreeList = FreeList::new();

/// End of the virtual memory address space reserved for kernel memory (4 GiB).
/// This also marks the start of the virtual memory address space reserved for the task heap.
const KERNEL_VIRTUAL_MEMORY_END: usize = 0x1_0000_0000;

/// End of the virtual memory address space reserved for task memory (128 TiB).
/// This is the maximum contiguous virtual memory area possible with current x86-64 CPUs, which only support 48-bit
/// linear addressing (in two 47-bit areas).
const TASK_VIRTUAL_MEMORY_END: usize = 0x8000_0000_0000;

pub fn init() {
	unimplemented!()
}

pub fn allocate(size: usize) -> usize {
	unimplemented!()
}

pub fn deallocate(virtual_address: usize, size: usize) {
	unimplemented!()
}

pub fn reserve(virtual_address: usize, size: usize) {
	unimplemented!()
}

pub fn print_information() {
	unsafe {
		KERNEL_FREE_LIST.print_information(" KERNEL VIRTUAL MEMORY FREE LIST ");
	}
}

#[inline]
pub fn task_heap_start() -> usize {
	KERNEL_VIRTUAL_MEMORY_END
}

#[inline]
pub fn task_heap_end() -> usize {
	TASK_VIRTUAL_MEMORY_END
}
