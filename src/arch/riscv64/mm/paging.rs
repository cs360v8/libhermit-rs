// Copyright (c) 2018 Colin Finck, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use core::marker::PhantomData;
use core::{fmt, ptr, usize};

use crate::arch::riscv64::kernel::percore::*;
use crate::arch::riscv64::kernel::processor;
use crate::arch::riscv64::mm::physicalmem;
use crate::arch::riscv64::mm::virtualmem;
//use crate::arch::riscv64::mm::{PAddr, VAddr};
use crate::mm;
use crate::scheduler;
use core::ops;
use core::hash::{Hash, Hasher};



extern "C" {
	#[linkage = "extern_weak"]
	static runtime_osinit: *const u8;
}


#[panic_handler]
pub fn alloc_error_handler() {
	unimplemented!()
}

#[alloc_error_handler]
pub fn panic_handler() {
	unimplemented!()
}


/// Log2 of base page size (12 bits).
pub const BASE_PAGE_SHIFT: usize = 12;

/// Size of a base page (4 KiB)
pub const BASE_PAGE_SIZE: usize = 4096;

/// Size of a large page (2 MiB)
pub const LARGE_PAGE_SIZE: usize = 1024 * 1024 * 2;

/// Align address downwards.
///
/// Returns the greatest x with alignment `align` so that x <= addr.
/// The alignment must be a power of 2.
#[inline(always)]
fn align_down(addr: u64, align: u64) -> u64 {
    addr & !(align - 1)
}

/// Align address upwards.
///
/// Returns the smallest x with alignment `align` so that x >= addr.
/// The alignment must be a power of 2.
#[inline(always)]
fn align_up(addr: u64, align: u64) -> u64 {
    let align_mask = align - 1;
    if addr & align_mask == 0 {
        addr
    } else {
        (addr | align_mask) + 1
    }
}


/// A wrapper for a physical address, which is in principle
/// derived from the crate x86.
#[repr(transparent)]
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct PAddr(pub u64);

impl PAddr {
    /// Convert to `u64`
    pub fn as_u64(self) -> u64 {
        self.0
    }

    /// Convert to `usize`
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }

    /// Physical Address zero.
    pub const fn zero() -> Self {
        PAddr(0)
    }

    /// Is zero?
    pub fn is_zero(self) -> bool {
        self == PAddr::zero()
    }

    fn align_up<U>(self, align: U) -> Self
    where
        U: Into<u64>,
    {
        PAddr(align_up(self.0, align.into()))
    }

    fn align_down<U>(self, align: U) -> Self
    where
        U: Into<u64>,
    {
        PAddr(align_down(self.0, align.into()))
    }

    /// Is this address aligned to `align`?
    ///
    /// # Note
    /// `align` must be a power of two.
    pub fn is_aligned<U>(self, align: U) -> bool
    where
        U: Into<u64> + Copy,
    {
        if !align.into().is_power_of_two() {
            return false;
        }

        self.align_down(align) == self
    }
}

impl From<u64> for PAddr {
    fn from(num: u64) -> Self {
        PAddr(num)
    }
}

impl From<usize> for PAddr {
    fn from(num: usize) -> Self {
        PAddr(num as u64)
    }
}

impl From<i32> for PAddr {
    fn from(num: i32) -> Self {
        PAddr(num as u64)
    }
}

impl Into<u64> for PAddr {
    fn into(self) -> u64 {
        self.0
    }
}

impl Into<usize> for PAddr {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl ops::Add for PAddr {
    type Output = PAddr;

    fn add(self, rhs: PAddr) -> Self::Output {
        PAddr(self.0 + rhs.0)
    }
}

impl ops::Add<u64> for PAddr {
    type Output = PAddr;

    fn add(self, rhs: u64) -> Self::Output {
        PAddr::from(self.0 + rhs)
    }
}

impl ops::Add<usize> for PAddr {
    type Output = PAddr;

    fn add(self, rhs: usize) -> Self::Output {
        PAddr::from(self.0 + rhs as u64)
    }
}

impl ops::AddAssign for PAddr {
    fn add_assign(&mut self, other: PAddr) {
        *self = PAddr::from(self.0 + other.0);
    }
}

impl ops::AddAssign<u64> for PAddr {
    fn add_assign(&mut self, offset: u64) {
        *self = PAddr::from(self.0 + offset);
    }
}

impl ops::Sub for PAddr {
    type Output = PAddr;

    fn sub(self, rhs: PAddr) -> Self::Output {
        PAddr::from(self.0 - rhs.0)
    }
}

impl ops::Sub<u64> for PAddr {
    type Output = PAddr;

    fn sub(self, rhs: u64) -> Self::Output {
        PAddr::from(self.0 - rhs)
    }
}

impl ops::Sub<usize> for PAddr {
    type Output = PAddr;

    fn sub(self, rhs: usize) -> Self::Output {
        PAddr::from(self.0 - rhs as u64)
    }
}

impl ops::Rem for PAddr {
    type Output = PAddr;

    fn rem(self, rhs: PAddr) -> Self::Output {
        PAddr(self.0 % rhs.0)
    }
}

impl ops::Rem<u64> for PAddr {
    type Output = u64;

    fn rem(self, rhs: u64) -> Self::Output {
        self.0 % rhs
    }
}

impl ops::Rem<usize> for PAddr {
    type Output = u64;

    fn rem(self, rhs: usize) -> Self::Output {
        self.0 % (rhs as u64)
    }
}

impl ops::BitAnd for PAddr {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        PAddr(self.0 & rhs.0)
    }
}

impl ops::BitAnd<u64> for PAddr {
    type Output = u64;

    fn bitand(self, rhs: u64) -> Self::Output {
        Into::<u64>::into(self) & rhs
    }
}

impl ops::BitOr for PAddr {
    type Output = PAddr;

    fn bitor(self, rhs: PAddr) -> Self::Output {
        PAddr(self.0 | rhs.0)
    }
}

impl ops::BitOr<u64> for PAddr {
    type Output = u64;

    fn bitor(self, rhs: u64) -> Self::Output {
        self.0 | rhs
    }
}

impl ops::Shr<u64> for PAddr {
    type Output = u64;

    fn shr(self, rhs: u64) -> Self::Output {
        self.0 >> rhs
    }
}

impl fmt::Binary for PAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for PAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Debug for PAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl fmt::LowerHex for PAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Octal for PAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::UpperHex for PAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Pointer for PAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use core::fmt::LowerHex;
        self.0.fmt(f)
    }
}

impl Hash for PAddr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// A wrapper for a virtual address, which is in principle
/// derived from the crate x86.
#[repr(transparent)]
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct VAddr(pub u64);

impl VAddr {
    /// Convert from `u64`
    pub const fn from_u64(v: u64) -> Self {
        VAddr(v)
    }

    /// Convert from `usize`
    pub const fn from_usize(v: usize) -> Self {
        VAddr(v as u64)
    }

    /// Convert to `u64`
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Convert to `usize`
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    /// Convert to mutable pointer.
    pub fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }

    /// Convert to pointer.
    pub fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }

    /// Physical Address zero.
    pub const fn zero() -> Self {
        VAddr(0)
    }

    /// Is zero?
    pub fn is_zero(self) -> bool {
        self == VAddr::zero()
    }

    fn align_up<U>(self, align: U) -> Self
    where
        U: Into<u64>,
    {
        VAddr(align_up(self.0, align.into()))
    }

    fn align_down<U>(self, align: U) -> Self
    where
        U: Into<u64>,
    {
        VAddr(align_down(self.0, align.into()))
    }

    /// Offset within the 4 KiB page.
    pub fn base_page_offset(self) -> u64 {
        self.0 & (BASE_PAGE_SIZE as u64 - 1)
    }

    /// Offset within the 2 MiB page.
    pub fn large_page_offset(self) -> u64 {
        self.0 & (LARGE_PAGE_SIZE as u64 - 1)
    }

    /// Return address of nearest 4 KiB page (lower or equal than self).
    pub fn align_down_to_base_page(self) -> Self {
        self.align_down(BASE_PAGE_SIZE as u64)
    }

    /// Return address of nearest 2 MiB page (lower or equal than self).
    pub fn align_down_to_large_page(self) -> Self {
        self.align_down(LARGE_PAGE_SIZE as u64)
    }

    /// Return address of nearest 4 KiB page (higher or equal than self).
    pub fn align_up_to_base_page(self) -> Self {
        self.align_up(BASE_PAGE_SIZE as u64)
    }

    /// Return address of nearest 2 MiB page (higher or equal than self).
    pub fn align_up_to_large_page(self) -> Self {
        self.align_up(LARGE_PAGE_SIZE as u64)
    }

    /// Is this address aligned to a 4 KiB page?
    pub fn is_base_page_aligned(self) -> bool {
        self.align_down(BASE_PAGE_SIZE as u64) == self
    }

    /// Is this address aligned to a 2 MiB page?
    pub fn is_large_page_aligned(self) -> bool {
        self.align_down(LARGE_PAGE_SIZE as u64) == self
    }

    /// Is this address aligned to `align`?
    ///
    /// # Note
    /// `align` must be a power of two.
    pub fn is_aligned<U>(self, align: U) -> bool
    where
        U: Into<u64> + Copy,
    {
        if !align.into().is_power_of_two() {
            return false;
        }

        self.align_down(align) == self
    }
}

impl From<u64> for VAddr {
    fn from(num: u64) -> Self {
        VAddr(num)
    }
}

impl From<i32> for VAddr {
    fn from(num: i32) -> Self {
        VAddr(num as u64)
    }
}

impl Into<u64> for VAddr {
    fn into(self) -> u64 {
        self.0
    }
}

impl From<usize> for VAddr {
    fn from(num: usize) -> Self {
        VAddr(num as u64)
    }
}

impl Into<usize> for VAddr {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl ops::Add for VAddr {
    type Output = VAddr;

    fn add(self, rhs: VAddr) -> Self::Output {
        VAddr(self.0 + rhs.0)
    }
}

impl ops::Add<u64> for VAddr {
    type Output = VAddr;

    fn add(self, rhs: u64) -> Self::Output {
        VAddr(self.0 + rhs)
    }
}

impl ops::Add<usize> for VAddr {
    type Output = VAddr;

    fn add(self, rhs: usize) -> Self::Output {
        VAddr::from(self.0 + rhs as u64)
    }
}

impl ops::AddAssign for VAddr {
    fn add_assign(&mut self, other: VAddr) {
        *self = VAddr::from(self.0 + other.0);
    }
}

impl ops::AddAssign<u64> for VAddr {
    fn add_assign(&mut self, offset: u64) {
        *self = VAddr::from(self.0 + offset);
    }
}

impl ops::AddAssign<usize> for VAddr {
    fn add_assign(&mut self, offset: usize) {
        *self = VAddr::from(self.0 + offset as u64);
    }
}

impl ops::Sub for VAddr {
    type Output = VAddr;

    fn sub(self, rhs: VAddr) -> Self::Output {
        VAddr::from(self.0 - rhs.0)
    }
}

impl ops::Sub<u64> for VAddr {
    type Output = VAddr;

    fn sub(self, rhs: u64) -> Self::Output {
        VAddr::from(self.0 - rhs)
    }
}

impl ops::Sub<usize> for VAddr {
    type Output = VAddr;

    fn sub(self, rhs: usize) -> Self::Output {
        VAddr::from(self.0 - rhs as u64)
    }
}

impl ops::Rem for VAddr {
    type Output = VAddr;

    fn rem(self, rhs: VAddr) -> Self::Output {
        VAddr(self.0 % rhs.0)
    }
}

impl ops::Rem<u64> for VAddr {
    type Output = u64;

    fn rem(self, rhs: Self::Output) -> Self::Output {
        self.0 % rhs
    }
}

impl ops::Rem<usize> for VAddr {
    type Output = usize;

    fn rem(self, rhs: Self::Output) -> Self::Output {
        self.as_usize() % rhs
    }
}

impl ops::BitAnd for VAddr {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        VAddr(self.0 & rhs.0)
    }
}

impl ops::BitAnd<u64> for VAddr {
    type Output = VAddr;

    fn bitand(self, rhs: u64) -> Self::Output {
        VAddr(self.0 & rhs)
    }
}

impl ops::BitAnd<usize> for VAddr {
    type Output = VAddr;

    fn bitand(self, rhs: usize) -> Self::Output {
        VAddr(self.0 & rhs as u64)
    }
}

impl ops::BitAnd<i32> for VAddr {
    type Output = VAddr;

    fn bitand(self, rhs: i32) -> Self::Output {
        VAddr(self.0 & rhs as u64)
    }
}

impl ops::BitOr for VAddr {
    type Output = VAddr;

    fn bitor(self, rhs: VAddr) -> VAddr {
        VAddr(self.0 | rhs.0)
    }
}

impl ops::BitOr<u64> for VAddr {
    type Output = VAddr;

    fn bitor(self, rhs: u64) -> Self::Output {
        VAddr(self.0 | rhs)
    }
}

impl ops::BitOr<usize> for VAddr {
    type Output = VAddr;

    fn bitor(self, rhs: usize) -> Self::Output {
        VAddr(self.0 | rhs as u64)
    }
}

impl ops::Shr<u64> for VAddr {
    type Output = u64;

    fn shr(self, rhs: u64) -> Self::Output {
        self.0 >> rhs as u64
    }
}

impl ops::Shr<usize> for VAddr {
    type Output = u64;

    fn shr(self, rhs: usize) -> Self::Output {
        self.0 >> rhs as u64
    }
}

impl ops::Shr<i32> for VAddr {
    type Output = u64;

    fn shr(self, rhs: i32) -> Self::Output {
        self.0 >> rhs as u64
    }
}

impl fmt::Binary for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl fmt::Debug for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl fmt::LowerHex for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Octal for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::UpperHex for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Pointer for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use core::fmt::LowerHex;
        self.0.fmt(f)
    }
}

impl Hash for VAddr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// Pointer to the root page table (called "Level 0" in ARM terminology).
/// Setting the upper bits to zero tells the MMU to use TTBR0 for the base address for the first table.
///
/// See entry.S and ARM Cortex-A Series Programmer's Guide for ARMv8-A, Version 1.0, PDF page 172
const L0TABLE_ADDRESS: *mut PageTable<L0Table> = 0x0000_FFFF_FFFF_F000 as *mut PageTable<L0Table>;

/// Number of Offset bits of a virtual address for a 4 KiB page, which are shifted away to get its Page Frame Number (PFN).
const PAGE_BITS: usize = 12;

/// Number of bits of the index in each table (L0Table, L1Table, L2Table, L3Table).
const PAGE_MAP_BITS: usize = 9;

/// A mask where PAGE_MAP_BITS are set to calculate a table index.
const PAGE_MAP_MASK: usize = 0x1FF;

bitflags! {
	/// Useful flags for an entry in either table (L0Table, L1Table, L2Table, L3Table).
	///
	/// See ARM Architecture Reference Manual, ARMv8, for ARMv8-A Reference Profile, Issue C.a, Chapter D4.3.3
	pub struct PageTableEntryFlags: usize {
		/// Set if this entry is valid.
		const PRESENT = 1 << 0;

		/// Set if this entry points to a table or a 4 KiB page.
		const TABLE_OR_4KIB_PAGE = 1 << 1;

		/// Set if this entry points to device memory (non-gathering, non-reordering, no early write acknowledgement)
		const DEVICE_NGNRNE = 0 << 4 | 0 << 3 | 0 << 2;

		/// Set if this entry points to device memory (non-gathering, non-reordering, early write acknowledgement)
		const DEVICE_NGNRE = 0 << 4 | 0 << 3 | 1 << 2;

		/// Set if this entry points to device memory (gathering, reordering, early write acknowledgement)
		const DEVICE_GRE = 0 << 4 | 1 << 3 | 0 << 2;

		/// Set if this entry points to normal memory (non-cacheable)
		const NORMAL_NC = 0 << 4 | 1 << 3 | 1 << 2;

		/// Set if this entry points to normal memory (cacheable)
		const NORMAL = 1 << 4 | 0 << 3 | 0 << 2;

		/// Set if memory referenced by this entry shall be read-only.
		const READ_ONLY = 1 << 7;

		/// Set if this entry shall be shared between all cores of the system.
		const INNER_SHAREABLE = 1 << 8 | 1 << 9;

		/// Set if software has accessed this entry (for memory access or address translation).
		const ACCESSED = 1 << 10;

		/// Set if code execution shall be disabled for memory referenced by this entry in privileged mode.
		const PRIVILEGED_EXECUTE_NEVER = 1 << 53;

		/// Set if code execution shall be disabled for memory referenced by this entry in unprivileged mode.
		const UNPRIVILEGED_EXECUTE_NEVER = 1 << 54;
	}
}

impl PageTableEntryFlags {
	/// An empty set of flags for unused/zeroed table entries.
	/// Needed as long as empty() is no const function.
	const BLANK: PageTableEntryFlags = PageTableEntryFlags { bits: 0 };

	pub fn device(&mut self) -> &mut Self {
		self.insert(PageTableEntryFlags::DEVICE_NGNRE);
		self
	}

	pub fn normal(&mut self) -> &mut Self {
		self.insert(PageTableEntryFlags::NORMAL);
		self
	}

	pub fn read_only(&mut self) -> &mut Self {
		self.insert(PageTableEntryFlags::READ_ONLY);
		self
	}

	pub fn writable(&mut self) -> &mut Self {
		self.remove(PageTableEntryFlags::READ_ONLY);
		self
	}

	pub fn execute_disable(&mut self) -> &mut Self {
		self.insert(PageTableEntryFlags::PRIVILEGED_EXECUTE_NEVER);
		self.insert(PageTableEntryFlags::UNPRIVILEGED_EXECUTE_NEVER);
		self
	}
}

/// An entry in either table
#[derive(Clone, Copy)]
pub struct PageTableEntry {
	/// Physical memory address this entry refers, combined with flags from PageTableEntryFlags.
	physical_address_and_flags: usize,
}

impl PageTableEntry {
	/// Return the stored physical address.
	pub fn address(&self) -> usize {
		self.physical_address_and_flags & !(BasePageSize::SIZE - 1) & !(usize::MAX << 48)
	}

	/// Returns whether this entry is valid (present).
	fn is_present(&self) -> bool {
		(self.physical_address_and_flags & PageTableEntryFlags::PRESENT.bits()) != 0
	}

	/// Mark this as a valid (present) entry and set address translation and flags.
	///
	/// # Arguments
	///
	/// * `physical_address` - The physical memory address this entry shall translate to
	/// * `flags` - Flags from PageTableEntryFlags (note that the PRESENT, INNER_SHAREABLE, and ACCESSED flags are set automatically)
	fn set(&mut self, physical_address: usize, flags: PageTableEntryFlags) {
		// Verify that the offset bits for a 4 KiB page are zero.
		assert_eq!(
			physical_address % BasePageSize::SIZE,
			0,
			"Physical address is not on a 4 KiB page boundary (physical_address = {:#X})",
			physical_address
		);

		let mut flags_to_set = flags;
		flags_to_set.insert(PageTableEntryFlags::PRESENT);
		flags_to_set.insert(PageTableEntryFlags::INNER_SHAREABLE);
		flags_to_set.insert(PageTableEntryFlags::ACCESSED);
		self.physical_address_and_flags = physical_address | flags_to_set.bits();
	}
}

/// A generic interface to support all possible page sizes.
///
/// This is defined as a subtrait of Copy to enable #[derive(Clone, Copy)] for Page.
/// Currently, deriving implementations for these traits only works if all dependent types implement it as well.
pub trait PageSize: Copy {
	/// The page size in bytes.
	const SIZE: usize;

	/// The page table level at which a page of this size is mapped
	const MAP_LEVEL: usize;

	/// Any extra flag that needs to be set to map a page of this size.
	/// For example: PageTableEntryFlags::TABLE_OR_4KIB_PAGE.
	const MAP_EXTRA_FLAG: PageTableEntryFlags;
}

/// A 4 KiB page mapped in the L3Table.
#[derive(Clone, Copy)]
pub enum BasePageSize {}
impl PageSize for BasePageSize {
	const SIZE: usize = 4096;
	const MAP_LEVEL: usize = 3;
	const MAP_EXTRA_FLAG: PageTableEntryFlags = PageTableEntryFlags::TABLE_OR_4KIB_PAGE;
}

/// A 2 MiB page mapped in the L2Table.
#[derive(Clone, Copy)]
pub enum LargePageSize {}
impl PageSize for LargePageSize {
	const SIZE: usize = 2 * 1024 * 1024;
	const MAP_LEVEL: usize = 2;
	const MAP_EXTRA_FLAG: PageTableEntryFlags = PageTableEntryFlags::BLANK;
}

/// A 1 GiB page mapped in the L1Table.
#[derive(Clone, Copy)]
pub enum HugePageSize {}
impl PageSize for HugePageSize {
	const SIZE: usize = 1024 * 1024 * 1024;
	const MAP_LEVEL: usize = 1;
	const MAP_EXTRA_FLAG: PageTableEntryFlags = PageTableEntryFlags::BLANK;
}

/// A memory page of the size given by S.
#[derive(Clone, Copy)]
struct Page<S: PageSize> {
	/// Virtual memory address of this page.
	/// This is rounded to a page size boundary on creation.
	virtual_address: usize,

	/// Required by Rust to support the S parameter.
	size: PhantomData<S>,
}

impl<S: PageSize> Page<S> {
	/// Return the stored virtual address.
	fn address(&self) -> usize {
		self.virtual_address
	}

	/// Flushes this page from the TLB of this CPU.
	fn flush_from_tlb(&self) {
		// See ARM Cortex-A Series Programmer's Guide for ARMv8-A, Version 1.0, PDF page 198
		//
		// We use "vale1is" instead of "vae1is" to always flush the last table level only (performance optimization).
		// The "is" attribute broadcasts the TLB flush to all cores, so we don't need an IPI (unlike x86_64).
		unsafe {
			llvm_asm!("dsb ishst; tlbi vale1is, $0; dsb ish; isb" :: "r"(self.virtual_address) : "memory" : "volatile");
		}
	}

	/// Returns whether the given virtual address is a valid one in the Riscv64 memory model.
	///
	/// Current Riscv64 supports only 48-bit for virtual memory addresses.
	/// The upper bits must always be 0 or 1 and indicate whether TBBR0 or TBBR1 contains the
	/// base address. So always enforce 0 here.
	fn is_valid_address(virtual_address: usize) -> bool {
		virtual_address < 0x1_0000_0000_0000
	}

	/// Returns a Page including the given virtual address.
	/// That means, the address is rounded down to a page size boundary.
	fn including_address(virtual_address: usize) -> Self {
		assert!(
			Self::is_valid_address(virtual_address),
			"Virtual address {:#X} is invalid",
			virtual_address
		);

		Self {
			virtual_address: align_down!(virtual_address, S::SIZE),
			size: PhantomData,
		}
	}

	/// Returns a PageIter to iterate from the given first Page to the given last Page (inclusive).
	fn range(first: Self, last: Self) -> PageIter<S> {
		assert!(first.virtual_address <= last.virtual_address);
		PageIter {
			current: first,
			last: last,
		}
	}

	/// Returns the index of this page in the table given by L.
	fn table_index<L: PageTableLevel>(&self) -> usize {
		assert!(L::LEVEL <= S::MAP_LEVEL);
		self.virtual_address >> PAGE_BITS >> (3 - L::LEVEL) * PAGE_MAP_BITS & PAGE_MAP_MASK
	}
}

/// An iterator to walk through a range of pages of size S.
struct PageIter<S: PageSize> {
	current: Page<S>,
	last: Page<S>,
}

impl<S: PageSize> Iterator for PageIter<S> {
	type Item = Page<S>;

	fn next(&mut self) -> Option<Page<S>> {
		if self.current.virtual_address <= self.last.virtual_address {
			let p = self.current;
			self.current.virtual_address += S::SIZE;
			Some(p)
		} else {
			None
		}
	}
}

/// An interface to allow for a generic implementation of struct PageTable for all 4 page tables.
/// Must be implemented by all page tables.
trait PageTableLevel {
	/// Numeric page table level
	const LEVEL: usize;
}

/// An interface for page tables with sub page tables (all except L3Table).
/// Having both PageTableLevel and PageTableLevelWithSubtables leverages Rust's typing system to provide
/// a subtable method only for those that have sub page tables.
///
/// Kudos to Philipp Oppermann for the trick!
trait PageTableLevelWithSubtables: PageTableLevel {
	type SubtableLevel;
}

/// The Level 0 Table
enum L0Table {}
impl PageTableLevel for L0Table {
	const LEVEL: usize = 0;
}

impl PageTableLevelWithSubtables for L0Table {
	type SubtableLevel = L1Table;
}

/// The Level 1 Table (can map 1 GiB pages)
enum L1Table {}
impl PageTableLevel for L1Table {
	const LEVEL: usize = 1;
}

impl PageTableLevelWithSubtables for L1Table {
	type SubtableLevel = L2Table;
}

/// The Level 2 Table (can map 2 MiB pages)
enum L2Table {}
impl PageTableLevel for L2Table {
	const LEVEL: usize = 2;
}

impl PageTableLevelWithSubtables for L2Table {
	type SubtableLevel = L3Table;
}

/// The Level 3 Table (can map 4 KiB pages)
enum L3Table {}
impl PageTableLevel for L3Table {
	const LEVEL: usize = 3;
}

/// Representation of any page table in memory.
/// Parameter L supplies information for Rust's typing system to distinguish between the different tables.
struct PageTable<L> {
	/// Each page table has 512 entries (can be calculated using PAGE_MAP_BITS).
	entries: [PageTableEntry; 1 << PAGE_MAP_BITS],

	/// Required by Rust to support the L parameter.
	level: PhantomData<L>,
}

/// A trait defining methods every page table has to implement.
/// This additional trait is necessary to make use of Rust's specialization feature and provide a default
/// implementation of some methods.
trait PageTableMethods {
	fn get_page_table_entry<S: PageSize>(&self, page: Page<S>) -> Option<PageTableEntry>;
	fn map_page_in_this_table<S: PageSize>(
		&mut self,
		page: Page<S>,
		physical_address: usize,
		flags: PageTableEntryFlags,
	);
	fn map_page<S: PageSize>(
		&mut self,
		page: Page<S>,
		physical_address: usize,
		flags: PageTableEntryFlags,
	);
}

impl<L: PageTableLevel> PageTableMethods for PageTable<L> {
	/// Maps a single page in this table to the given physical address.
	///
	/// Must only be called if a page of this size is mapped at this page table level!
	fn map_page_in_this_table<S: PageSize>(
		&mut self,
		page: Page<S>,
		physical_address: usize,
		flags: PageTableEntryFlags,
	) {
		assert_eq!(L::LEVEL, S::MAP_LEVEL);
		let index = page.table_index::<L>();
		let flush = self.entries[index].is_present();

		self.entries[index].set(physical_address, S::MAP_EXTRA_FLAG | flags);

		if flush {
			page.flush_from_tlb();
		}
	}

	/// Returns the PageTableEntry for the given page if it is present, otherwise returns None.
	///
	/// This is the default implementation called only for L3Table.
	/// It is overridden by a specialized implementation for all tables with sub tables (all except L3Table).
	default fn get_page_table_entry<S: PageSize>(&self, page: Page<S>) -> Option<PageTableEntry> {
		assert_eq!(L::LEVEL, S::MAP_LEVEL);
		let index = page.table_index::<L>();

		if self.entries[index].is_present() {
			Some(self.entries[index])
		} else {
			None
		}
	}

	/// Maps a single page to the given physical address.
	///
	/// This is the default implementation that just calls the map_page_in_this_table method.
	/// It is overridden by a specialized implementation for all tables with sub tables (all except L3Table).
	default fn map_page<S: PageSize>(
		&mut self,
		page: Page<S>,
		physical_address: usize,
		flags: PageTableEntryFlags,
	) {
		self.map_page_in_this_table::<S>(page, physical_address, flags)
	}
}

impl<L: PageTableLevelWithSubtables> PageTableMethods for PageTable<L>
where
	L::SubtableLevel: PageTableLevel,
{
	/// Returns the PageTableEntry for the given page if it is present, otherwise returns None.
	///
	/// This is the implementation for all tables with subtables (L0Table, L1Table, L2Table).
	/// It overrides the default implementation above.
	fn get_page_table_entry<S: PageSize>(&self, page: Page<S>) -> Option<PageTableEntry> {
		assert!(L::LEVEL <= S::MAP_LEVEL);
		let index = page.table_index::<L>();

		if self.entries[index].is_present() {
			if L::LEVEL < S::MAP_LEVEL {
				let subtable = self.subtable::<S>(page);
				subtable.get_page_table_entry::<S>(page)
			} else {
				Some(self.entries[index])
			}
		} else {
			None
		}
	}

	/// Maps a single page to the given physical address.
	///
	/// This is the implementation for all tables with subtables (L0Table, L1Table, L2Table).
	/// It overrides the default implementation above.
	fn map_page<S: PageSize>(
		&mut self,
		page: Page<S>,
		physical_address: usize,
		flags: PageTableEntryFlags,
	) {
		assert!(L::LEVEL <= S::MAP_LEVEL);

		if L::LEVEL < S::MAP_LEVEL {
			let index = page.table_index::<L>();

			// Does the table exist yet?
			if !self.entries[index].is_present() {
				// Allocate a single 4 KiB page for the new entry and mark it as a valid, writable subtable.
				let physical_address = physicalmem::allocate(BasePageSize::SIZE);
				self.entries[index].set(
					physical_address,
					PageTableEntryFlags::NORMAL | PageTableEntryFlags::TABLE_OR_4KIB_PAGE,
				);

				// Mark all entries as unused in the newly created table.
				let subtable = self.subtable::<S>(page);
				for entry in subtable.entries.iter_mut() {
					entry.physical_address_and_flags = 0;
				}
			}

			let subtable = self.subtable::<S>(page);
			subtable.map_page::<S>(page, physical_address, flags)
		} else {
			// Calling the default implementation from a specialized one is not supported (yet),
			// so we have to resort to an extra function.
			self.map_page_in_this_table::<S>(page, physical_address, flags)
		}
	}
}

impl<L: PageTableLevelWithSubtables> PageTable<L>
where
	L::SubtableLevel: PageTableLevel,
{
	/// Returns the next subtable for the given page in the page table hierarchy.
	///
	/// Must only be called if a page of this size is mapped in a subtable!
	fn subtable<S: PageSize>(&self, page: Page<S>) -> &mut PageTable<L::SubtableLevel> {
		assert!(L::LEVEL < S::MAP_LEVEL);

		// Calculate the address of the subtable.
		let index = page.table_index::<L>();
		let table_address = self as *const PageTable<L> as usize;
		let subtable_address =
			(table_address << PAGE_MAP_BITS) & !(usize::MAX << 48) | (index << PAGE_BITS);
		unsafe { &mut *(subtable_address as *mut PageTable<L::SubtableLevel>) }
	}

	/// Maps a continuous range of pages.
	///
	/// # Arguments
	///
	/// * `range` - The range of pages of size S
	/// * `physical_address` - First physical address to map these pages to
	/// * `flags` - Flags from PageTableEntryFlags to set for the page table entry (e.g. WRITABLE or EXECUTE_DISABLE).
	///             The PRESENT and ACCESSED are already set automatically.
	fn map_pages<S: PageSize>(
		&mut self,
		range: PageIter<S>,
		physical_address: usize,
		flags: PageTableEntryFlags,
	) {
		let mut current_physical_address = physical_address;

		for page in range {
			self.map_page::<S>(page, current_physical_address, flags);
			current_physical_address += S::SIZE;
		}
	}
}

#[inline]
fn get_page_range<S: PageSize>(virtual_address: usize, count: usize) -> PageIter<S> {
	let first_page = Page::<S>::including_address(virtual_address);
	let last_page = Page::<S>::including_address(virtual_address + (count - 1) * S::SIZE);
	Page::range(first_page, last_page)
}

pub fn get_page_table_entry<S: PageSize>(virtual_address: usize) -> Option<PageTableEntry> {
	trace!("Looking up Page Table Entry for {:#X}", virtual_address);

	let page = Page::<S>::including_address(virtual_address);
	let root_pagetable = unsafe { &mut *L0TABLE_ADDRESS };
	root_pagetable.get_page_table_entry(page)
}

pub fn get_physical_address<S: PageSize>(virtual_address: usize) -> usize {
	trace!("Getting physical address for {:#X}", virtual_address);

	let page = Page::<S>::including_address(virtual_address);
	let root_pagetable = unsafe { &mut *L0TABLE_ADDRESS };
	let address = root_pagetable
		.get_page_table_entry(page)
		.expect("Entry not present")
		.address();
	let offset = virtual_address & (S::SIZE - 1);
	address | offset
}

/// Translate a virtual memory address to a physical one.
/// Just like get_physical_address, but automatically uses the correct page size for the respective memory address.
pub fn virtual_to_physical(virtual_address: usize) -> usize {
	if virtual_address < mm::kernel_start_address() {
		// Parts of the memory below the kernel image are identity-mapped.
		// However, this range should never be used in a virtual_to_physical call.
		panic!(
			"Trying to get the physical address of {:#X}, which is too low",
			virtual_address
		);
	} else if virtual_address < mm::kernel_end_address() {
		// The kernel image is mapped in 2 MiB pages.
		get_physical_address::<LargePageSize>(virtual_address)
	} else if virtual_address < virtualmem::task_heap_start() {
		// The kernel memory is mapped in 4 KiB pages.
		get_physical_address::<BasePageSize>(virtual_address)
	} else if virtual_address < virtualmem::task_heap_end() {
		// The application memory is mapped in 2 MiB pages.
		get_physical_address::<LargePageSize>(virtual_address)
	} else {
		// This range is currently unused by HermitCore.
		panic!(
			"Trying to get the physical address of {:#X}, which is too high",
			virtual_address
		);
	}
}

#[no_mangle]
pub extern "C" fn virt_to_phys(virtual_address: usize) -> usize {
	virtual_to_physical(virtual_address)
}

pub fn map<S: PageSize>(
	virtual_address: usize,
	physical_address: usize,
	count: usize,
	flags: PageTableEntryFlags,
) {
	trace!(
		"Mapping virtual address {:#X} to physical address {:#X} ({} pages)",
		virtual_address,
		physical_address,
		count
	);

	let range = get_page_range::<S>(virtual_address, count);
	let root_pagetable = unsafe { &mut *L0TABLE_ADDRESS };
	root_pagetable.map_pages(range, physical_address, flags);
}

#[inline]
pub fn get_application_page_size() -> usize {
	LargePageSize::SIZE
}

pub fn unmap<S: PageSize>(virtual_address: VAddr, count: usize) {
	unimplemented!()
}

pub fn identity_map(start_address: PAddr, end_address: PAddr) {
	unimplemented!()
}