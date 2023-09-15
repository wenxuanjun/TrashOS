use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::fmt::Debug;
use core::sync::atomic::{AtomicU64, Ordering};
use object::{File, Object, ObjectSegment};
use spin::RwLock;
use x86_64::structures::paging::PageTableFlags;
use x86_64::VirtAddr;

use super::scheduler;
use super::thread::Thread;
use crate::memory::create_page_table_from_kernel;
use crate::memory::GeneralPageTable;
use crate::memory::MemoryManager;

const KERNEL_PROCESS_NAME: &str = "kernel";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ProcessId(u64);

impl ProcessId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        ProcessId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct Process {
    id: ProcessId,
    pub name: Cow<'static, str>,
    pub page_table: GeneralPageTable,
    pub threads: VecDeque<Arc<RwLock<Box<Thread>>>>,
}

impl Process {
    fn new(name: &'static str) -> Box<Self> {
        let process = Process {
            id: ProcessId::new(),
            name: Cow::Borrowed(name),
            page_table: create_page_table_from_kernel(),
            threads: Default::default(),
        };
        Box::new(process)
    }

    pub fn new_kernel_process() -> Arc<RwLock<Box<Self>>> {
        let process = Arc::new(RwLock::new(Self::new(KERNEL_PROCESS_NAME)));
        scheduler::SCHEDULER.write().add(process.clone());
        process
    }

    pub fn new_user_process(
        name: &'static str,
        elf_raw_data: &'static [u8],
    ) -> Result<Arc<RwLock<Box<Self>>>, &'static str> {
        let process = Arc::new(RwLock::new(Self::new(name)));
        let binary = ProcessBinary::parse(elf_raw_data);
        Thread::new_user_thread(process.clone(), binary.entry() as usize);
        ProcessBinary::map_segments(&binary, &mut process.write().page_table)?;
        scheduler::SCHEDULER.write().add(process.clone());
        Ok(process)
    }
}

struct ProcessBinary;

impl ProcessBinary {
    fn parse(bin: &'static [u8]) -> File<'static> {
        if bin[0..4] != [0x7f, b'E', b'L', b'F'] {
            panic!("Expected ELF binary");
        }
        File::parse(bin).expect("Failed to parse ELF binary!")
    }

    fn map_segments(
        elf_file: &File,
        page_table: &mut GeneralPageTable,
    ) -> Result<(), &'static str> {
        unsafe {
            page_table.switch();
        }
        for segment in elf_file.segments() {
            let segment_start = VirtAddr::new(segment.address() as u64);
            let segment_end = segment_start + segment.size() as u64 - 1u64;

            let flags = PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::USER_ACCESSIBLE;

            <MemoryManager>::alloc_range(segment_start, segment_end, flags, page_table).unwrap();

            if let Ok(data) = segment.data() {
                let dest_ptr = segment_start.as_u64() as *mut u8;
                for (index, value) in data.iter().enumerate() {
                    unsafe {
                        core::ptr::write(dest_ptr.add(index), *value);
                    }
                }
            }
        }
        Ok(())
    }
}
