use alloc::string::String;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::fmt::Debug;
use core::sync::atomic::{AtomicU64, Ordering};
use object::{File, Object, ObjectSegment};
use spin::{Lazy, RwLock};
use x86_64::VirtAddr;
use x86_64::structures::paging::OffsetPageTable;

use super::thread::{SharedThread, Thread};
use crate::mem::{ExtendedPageTable, ref_current_page_table};
use crate::mem::{FRAME_ALLOCATOR, KERNEL_PAGE_TABLE};
use crate::mem::{MappingType, MemoryManager};

pub(super) type SharedProcess = Arc<RwLock<Process>>;
pub(super) type WeakSharedProcess = Weak<RwLock<Process>>;

pub static KERNEL_PROCESS: Lazy<SharedProcess> = Lazy::new(|| {
    let process = Process::new("kernel", ref_current_page_table());
    let process = Arc::new(RwLock::new(process));
    PROCESSES.write().push(process.clone());
    process
});

static PROCESSES: RwLock<Vec<SharedProcess>> = RwLock::new(Vec::new());

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcessId(pub u64);

impl ProcessId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        ProcessId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[allow(dead_code)]
pub struct Process {
    pub id: ProcessId,
    pub name: String,
    pub page_table: OffsetPageTable<'static>,
    pub threads: Vec<SharedThread>,
}

impl Process {
    pub fn new(name: &str, page_table: OffsetPageTable<'static>) -> Self {
        Self {
            id: ProcessId::new(),
            name: String::from(name),
            page_table,
            threads: Vec::new(),
        }
    }

    pub fn exit(&self) {
        let mut processes = PROCESSES.write();
        if let Some(index) = processes
            .iter()
            .position(|process| process.read().id == self.id)
        {
            processes.remove(index);
        }
    }

    pub fn create(name: &str, elf_data: &'static [u8]) {
        let binary = ProcessBinary::parse(elf_data);
        let mut page_table = unsafe { KERNEL_PAGE_TABLE.lock().deep_copy() };
        ProcessBinary::map_segments(&binary, &mut page_table);

        let process = Arc::new(RwLock::new(Self::new(name, page_table)));
        Thread::new_user_thread(Arc::downgrade(&process), binary.entry() as usize);
        PROCESSES.write().push(process.clone());
    }
}

struct ProcessBinary;

impl ProcessBinary {
    fn parse(bin: &'static [u8]) -> File<'static> {
        File::parse(bin).expect("Failed to parse ELF binary!")
    }

    fn map_segments(elf_file: &File, page_table: &mut OffsetPageTable<'static>) {
        for segment in elf_file.segments() {
            let address = VirtAddr::new(segment.address());

            MemoryManager::alloc_range(
                address,
                segment.size(),
                MappingType::UserCode.flags(),
                page_table,
            )
            .expect("Failed to allocate memory for ELF segment");

            if let Ok(data) = segment.data() {
                page_table.write_to_mapped_address(data, address);
            }
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        unsafe {
            self.page_table.free_user_page_table();
            log::info!("Process {} dropped", self.id.0);
            log::info!("Memory usage: {}", FRAME_ALLOCATOR.lock());
        }
    }
}
