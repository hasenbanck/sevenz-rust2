use ppmd_sys::{ISzAlloc, ISzAllocPtr};

pub(crate) struct Memory {
    inner: Box<MemoryInner>,
}

#[repr(C)]
struct MemoryInner {
    alloc: ISzAlloc,
    data: Vec<u8>,
}

impl Memory {
    pub(crate) fn new(size: u32) -> Self {
        let memory = Box::new(MemoryInner {
            alloc: ISzAlloc {
                Alloc: Some(Memory::alloc),
                Free: Some(Memory::free),
            },
            data: vec![0; size as usize],
        });

        Self { inner: memory }
    }

    pub(crate) fn allocation(&mut self) -> ISzAllocPtr {
        &mut self.inner.alloc
    }

    #[inline(always)]
    fn get_inner_memory<'a>(p: ISzAllocPtr) -> &'a mut MemoryInner {
        // Safety: This is safe because we make sure that `alloc` is the first field
        // of the `MemoryInner` and also `MemoryInner` is boxed and can't break out of it.
        unsafe { &mut *(p as *mut MemoryInner) }
    }

    unsafe extern "C" fn alloc(p: ISzAllocPtr, size: usize) -> *mut std::os::raw::c_void {
        let memory = Self::get_inner_memory(p);
        assert_eq!(size, memory.data.len());
        memory.data.as_mut_ptr() as *mut std::os::raw::c_void
    }

    unsafe extern "C" fn free(p: ISzAllocPtr, address: *mut std::os::raw::c_void) {
        if address.is_null() {
            return;
        }
        let memory = Self::get_inner_memory(p);
        assert_eq!(address.addr(), memory.data.as_mut_ptr().addr());
    }
}
