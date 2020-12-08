// Copyright (c) 2017 Colin Finck, RWTH Aachen University
//               2020 Thomas Lambertz, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::synch::spinlock::SpinlockIrqSave;
use alloc::vec::Vec;
use core::convert::TryInto;
use core::{fmt, u32, u8};

// TODO: should these be pub? currently needed since used in virtio.rs maybe use getter methods to be more flexible.
pub const PCI_MAX_BUS_NUMBER: u8 = 32;
pub const PCI_MAX_DEVICE_NUMBER: u8 = 32;

pub const PCI_CONFIG_ADDRESS_PORT: u16 = 0xCF8;
pub const PCI_CONFIG_ADDRESS_ENABLE: u32 = 1 << 31;

pub const PCI_CONFIG_DATA_PORT: u16 = 0xCFC;
pub const PCI_COMMAND_BUSMASTER: u32 = 1 << 2;

pub const PCI_ID_REGISTER: u32 = 0x00;
pub const PCI_COMMAND_REGISTER: u32 = 0x04;
pub const PCI_CLASS_REGISTER: u32 = 0x08;
pub const PCI_HEADER_REGISTER: u32 = 0x0C;
pub const PCI_BAR0_REGISTER: u32 = 0x10;
pub const PCI_CAPABILITY_LIST_REGISTER: u32 = 0x34;
pub const PCI_INTERRUPT_REGISTER: u32 = 0x3C;

pub const PCI_STATUS_CAPABILITIES_LIST: u32 = 1 << 4;

pub const PCI_BASE_ADDRESS_IO_SPACE: u32 = 1 << 0;
pub const PCI_MEM_BASE_ADDRESS_64BIT: u32 = 1 << 2;
pub const PCI_MEM_PREFETCHABLE: u32 = 1 << 3;
pub const PCI_MEM_BASE_ADDRESS_MASK: u32 = 0xFFFF_FFF0;
pub const PCI_IO_BASE_ADDRESS_MASK: u32 = 0xFFFF_FFFC;

pub const PCI_HEADER_TYPE_MASK: u32 = 0x007F_0000;
pub const PCI_MULTIFUNCTION_MASK: u32 = 0x0080_0000;

pub const PCI_CAP_ID_VNDR: u32 = 0x09;

static mut PCI_ADAPTERS: Vec<PciAdapter> = Vec::new();
static mut PCI_DRIVERS: Vec<PciDriver> = Vec::new();

/// Classes of PCI nodes.
#[allow(dead_code)]
#[derive(Copy, Clone, Debug, FromPrimitive, ToPrimitive, PartialEq)]
pub enum PciClassCode {
	TooOld = 0x00,
	MassStorage = 0x01,
	NetworkController = 0x02,
	DisplayController = 0x03,
	MultimediaController = 0x04,
	MemoryController = 0x05,
	BridgeDevice = 0x06,
	SimpleCommunicationController = 0x07,
	BaseSystemPeripheral = 0x08,
	InputDevice = 0x09,
	DockingStation = 0x0A,
	Processor = 0x0B,
	SerialBusController = 0x0C,
	WirelessController = 0x0D,
	IntelligentIoController = 0x0E,
	EncryptionController = 0x0F,
	DataAcquisitionSignalProcessing = 0x11,
	Other = 0xFF,
}

/// Network Controller Sub Classes
#[allow(dead_code)]
#[derive(Copy, Clone, Debug, FromPrimitive, ToPrimitive, PartialEq)]
pub enum PciNetworkControllerSubclass {
	EthernetController = 0x00,
	TokenRingController = 0x01,
	FDDIController = 0x02,
	ATMController = 0x03,
	ISDNController = 0x04,
	WorldFipController = 0x05,
	PICMGController = 0x06,
	InfinibandController = 0x07,
	FabricController = 0x08,
	NetworkController = 0x80,
}

#[derive(Clone, Debug)]
pub struct PciAdapter {
	pub bus: u8,
	pub device: u8,
	pub vendor_id: u16,
	pub device_id: u16,
	pub class_id: u8,
	pub subclass_id: u8,
	pub programming_interface_id: u8,
	pub base_addresses: Vec<PciBar>,
	pub irq: u8,
}
#[derive(Clone, Copy, Debug)]
pub enum PciBar {
	IO(IOBar),
	Memory(MemoryBar),
}
#[derive(Clone, Copy, Debug)]
pub struct IOBar {
	pub index: u8,
	pub addr: u32,
	pub size: usize,
}
#[derive(Clone, Copy, Debug)]
pub struct MemoryBar {
	pub index: u8,
	pub addr: usize,
	pub size: usize,
	pub width: u8, // 32 or 64 bit
	pub prefetchable: bool,
}

use core::marker::PhantomData;
pub struct PciDriver<'a> {
	test: PhantomData<'a>
}

pub fn register_driver(drv: PciDriver<'static>) {
	unsafe {
		PCI_DRIVERS.push(drv);
	}
}

pub fn get_network_driver() {
    unimplemented!()
}

pub fn get_filesystem_driver() {
    unimplemented!()
}

pub fn read_config(bus: u8, device: u8, register: u32) -> u32 {
    unimplemented!()
}

pub fn write_config(bus: u8, device: u8, register: u32, data: u32) {
    unimplemented!()
}

pub fn get_adapter(vendor_id: u16, device_id: u16) {
    unimplemented!()
}

pub fn init() {
    unimplemented!()
}

pub fn init_drivers() {
    unimplemented!()
}

pub fn print_information() {
    unimplemented!()
}
