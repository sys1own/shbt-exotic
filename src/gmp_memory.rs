//! Custom GMP/MPFR memory allocator for deterministic 512-bit arithmetic.
//!
//! The allocator is installed via `gmp_mpfr_sys::gmp::set_memory_functions` so
//! that `rug` limb allocations are served from a pre-resident arena instead of
//! the libc heap.  This removes allocator jitter from the high-frequency HIL
//! audit path.

use std::alloc::{alloc, dealloc, Layout};
use std::ffi::c_void;
use std::sync::{Mutex, Once};

const ARENA_SIZE: usize = 16 * 1024 * 1024;
const SIZE_CLASSES: [usize; 5] = [32, 64, 128, 256, 512];
const MAX_CLASS_SIZE: usize = SIZE_CLASSES[SIZE_CLASSES.len() - 1];
const NULL_OFFSET: usize = usize::MAX;

#[repr(C, align(16))]
struct Arena([u8; ARENA_SIZE]);

struct Allocator {
    arena: Arena,
    bump: usize,
    free_lists: [usize; SIZE_CLASSES.len()],
}

impl Allocator {
    const fn new() -> Self {
        Allocator {
            arena: Arena([0; ARENA_SIZE]),
            bump: 0,
            free_lists: [NULL_OFFSET; SIZE_CLASSES.len()],
        }
    }

    unsafe fn alloc(&mut self, size: usize) -> *mut u8 {
        if let Some(class) = size_class(size) {
            if self.free_lists[class] != NULL_OFFSET {
                let node = self.free_lists[class];
                self.free_lists[class] = read_usize(self.arena.0.as_mut_ptr().add(node));
                return self.arena.0.as_mut_ptr().add(node);
            }
            let aligned = (size + 15) & !15;
            if self.bump + aligned + 8 <= ARENA_SIZE {
                let ptr = self.arena.0.as_mut_ptr().add(self.bump + 8);
                self.bump += aligned + 8;
                return ptr;
            }
        }
        fallback_alloc(size)
    }

    unsafe fn free(&mut self, ptr: *mut u8, size: usize) {
        if ptr.is_null() {
            return;
        }
        if let Some(class) = size_class(size) {
            if ptr >= self.arena.0.as_mut_ptr()
                && ptr < self.arena.0.as_mut_ptr().add(ARENA_SIZE)
            {
                let node = ptr.offset_from(self.arena.0.as_mut_ptr()) as usize;
                write_usize(ptr, self.free_lists[class]);
                self.free_lists[class] = node;
                return;
            }
        }
        fallback_free(ptr, size);
    }

    unsafe fn realloc(&mut self, ptr: *mut u8, old_size: usize, new_size: usize) -> *mut u8 {
        if size_class(old_size) == size_class(new_size) && new_size <= MAX_CLASS_SIZE {
            return ptr;
        }
        let new_ptr = self.alloc(new_size);
        if !ptr.is_null() {
            let copy_size = old_size.min(new_size);
            std::ptr::copy_nonoverlapping(ptr, new_ptr, copy_size);
            self.free(ptr, old_size);
        }
        new_ptr
    }
}

static GMP_ALLOCATOR: Mutex<Allocator> = Mutex::new(Allocator::new());
static INIT: Once = Once::new();

pub fn init() {
    INIT.call_once(|| unsafe {
        gmp_mpfr_sys::gmp::set_memory_functions(
            Some(gmp_alloc),
            Some(gmp_realloc),
            Some(gmp_free),
        );
    });
}

extern "C" fn gmp_alloc(size: usize) -> *mut c_void {
    unsafe { GMP_ALLOCATOR.lock().unwrap().alloc(size) as *mut c_void }
}

unsafe extern "C" fn gmp_realloc(ptr: *mut c_void, old_size: usize, new_size: usize) -> *mut c_void {
    GMP_ALLOCATOR
        .lock()
        .unwrap()
        .realloc(ptr as *mut u8, old_size, new_size) as *mut c_void
}

unsafe extern "C" fn gmp_free(ptr: *mut c_void, size: usize) {
    GMP_ALLOCATOR.lock().unwrap().free(ptr as *mut u8, size);
}

fn size_class(size: usize) -> Option<usize> {
    SIZE_CLASSES.iter().position(|&c| size <= c)
}

unsafe fn read_usize(ptr: *mut u8) -> usize {
    std::ptr::read_unaligned(ptr as *const usize)
}

unsafe fn write_usize(ptr: *mut u8, value: usize) {
    std::ptr::write_unaligned(ptr as *mut usize, value);
}

unsafe fn fallback_alloc(size: usize) -> *mut u8 {
    let layout = Layout::from_size_align(size, 16).unwrap_or(Layout::from_size_align_unchecked(size, 8));
    let ptr = alloc(layout);
    if ptr.is_null() {
        std::alloc::handle_alloc_error(layout);
    }
    ptr
}

unsafe fn fallback_free(ptr: *mut u8, size: usize) {
    if ptr.is_null() {
        return;
    }
    let layout = Layout::from_size_align(size, 16).unwrap_or(Layout::from_size_align_unchecked(size, 8));
    dealloc(ptr, layout);
}
