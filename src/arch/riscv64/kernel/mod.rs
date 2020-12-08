// Copyright (c) 2018 Stefan Lankes, RWTH Aachen University
//                    Colin Finck, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

include!("../../../config.rs");

pub mod irq;
pub mod percore;
pub mod processor;
pub mod scheduler;
pub mod serial;
pub mod stubs;
pub mod systemtime;
pub mod pci;

use crate::arch::riscv64::kernel::percore::*;
use crate::arch::riscv64::kernel::serial::SerialPort;
pub use crate::arch::riscv64::kernel::stubs::*;
pub use crate::arch::riscv64::kernel::systemtime::get_boot_time;
use crate::environment;
use crate::kernel_message_buffer;
use crate::synch::spinlock::Spinlock;
use core::ptr;
use core::intrinsics::volatile_load;

const SERIAL_PORT_BAUDRATE: u32 = 115200;

// lazy_static! {
// 	static ref COM1: SerialPort = SerialPort::new(unsafe { BOOT_INFO.uartport });
// 	static ref CPU_ONLINE: Spinlock<&'static mut u32> =
// 		Spinlock::new(unsafe { &mut BOOT_INFO.cpu_online });
// }

#[repr(C)]
struct BootInfo {
	magic_number: u32,
	version: u32,
	base: u64,
	limit: u64,
	image_size: u64,
	current_stack_address: u64,
	current_percore_address: u64,
	host_logical_addr: u64,
	boot_gtod: u64,
	mb_info: u64,
	cmdline: u64,
	cmdsize: u64,
	cpu_freq: u32,
	boot_processor: u32,
	cpu_online: u32,
	possible_cpus: u32,
	current_boot_id: u32,
	uartport: u32,
	single_kernel: u8,
	uhyve: u8,
	hcip: [u8; 4],
	hcgateway: [u8; 4],
	hcmask: [u8; 4],
	boot_stack: [u8; KERNEL_STACK_SIZE],
}

/// Kernel header to announce machine features
#[link_section = ".mboot"]
static mut BOOT_INFO: BootInfo = BootInfo {
	magic_number: 0xC0DECAFEu32,
	version: 0,
	base: 0,
	limit: 0,
	image_size: 0,
	current_stack_address: 0,
	current_percore_address: 0,
	host_logical_addr: 0,
	boot_gtod: 0,
	mb_info: 0,
	cmdline: 0,
	cmdsize: 0,
	cpu_freq: 0,
	boot_processor: !0,
	cpu_online: 0,
	possible_cpus: 0,
	current_boot_id: 0,
	uartport: 0x9000000, // Initialize with QEMU's UART address
	single_kernel: 1,
	uhyve: 0,
	hcip: [10, 0, 5, 2],
	hcgateway: [10, 0, 5, 1],
	hcmask: [255, 255, 255, 0],
	boot_stack: [0xCD; KERNEL_STACK_SIZE],
};

// FUNCTIONS

pub fn get_image_size() -> usize {
	unsafe { volatile_load(&BOOT_INFO.image_size) as usize }
}

pub fn get_limit() -> usize {
	unsafe { volatile_load(&BOOT_INFO.limit) as usize }
}

pub fn get_mbinfo() -> usize {
	unsafe { volatile_load(&BOOT_INFO.mb_info) as usize }
}

pub fn get_processor_count() -> u32 {
	unsafe { volatile_load(&BOOT_INFO.cpu_online) as usize }
}

/// Whether HermitCore is running under the "uhyve" hypervisor.
pub fn is_uhyve() -> bool {
	unsafe { volatile_load(&BOOT_INFO.uhyve) != 0 }
}

/// Whether HermitCore is running alone (true) or side-by-side to Linux in Multi-Kernel mode (false).
pub fn is_single_kernel() -> bool {
	unsafe { volatile_load(&BOOT_INFO.single_kernel) != 0 }
}

pub fn get_cmdsize() -> usize {
	unsafe { volatile_load(&BOOT_INFO.cmdsize) as usize }
}

pub fn get_cmdline() -> usize {
	unsafe { volatile_load(&BOOT_INFO.cmdline) as usize }
}

/// Earliest initialization function called by the Boot Processor.
pub fn message_output_init() {
	unimplemented!()
}

pub fn output_message_byte(byte: u8) {
	unimplemented!()
}

pub fn output_message_buf(byte: &[u8]) {
	unimplemented!()
}

/// Real Boot Processor initialization as soon as we have put the first Welcome message on the screen.
pub fn boot_processor_init() {
	unimplemented!()
}

/// Boots all available Application Processors on bare-metal or QEMU.
/// Called after the Boot Processor has been fully initialized along with its scheduler.
pub fn boot_application_processors() {
	// Nothing to do here yet.
}

/// Application Processor initialization
pub fn application_processor_init() {
	percore::init();
	/*processor::configure();
	gdt::add_current_core();
	idt::install();
	apic::init_x2apic();
	apic::init_local_apic();
	irq::enable();*/
	finish_processor_init();
}

fn finish_processor_init() {
	unimplemented!()
}

pub fn network_adapter_init() -> i32 {
	// Riscv64 supports no network adapters on bare-metal/QEMU, so return a failure code.
	-1
}

pub fn print_statistics() {
	unimplemented!()
}