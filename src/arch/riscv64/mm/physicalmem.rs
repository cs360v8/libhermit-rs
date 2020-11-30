// Copyright (c) 2017 Colin Finck, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::arch::riscv64::mm::paging::{BasePageSize, PageSize};
use crate::mm;
use crate::mm::freelist::{FreeList, FreeListEntry};

extern "C" {
	static limit: usize;
}

static mut PHYSICAL_FREE_LIST: FreeList = FreeList::new();

fn detect_from_limits() -> Result<(), ()> {
	unimplemented!()
}

pub fn init() {
	detect_from_limits().unwrap();
}

pub fn init_page_tables() {}

pub fn allocate(size: usize) -> usize {
	unimplemented!()
}

pub fn total_memory_size() -> usize {
	unimplemented!()
}

pub fn allocate_aligned(size: usize, alignment: usize) -> usize {
	unimplemented!()
}

/// This function must only be called from mm::deallocate!
/// Otherwise, it may fail due to an empty node pool (POOL.maintain() is called in virtualmem::deallocate)
pub fn deallocate(physical_address: usize, size: usize) {
	unimplemented!()
}

pub fn print_information() {
	unsafe {
		PHYSICAL_FREE_LIST.print_information(" PHYSICAL MEMORY FREE LIST ");
	}
}
