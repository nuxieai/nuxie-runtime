//! Node: `cxx:Record:Luau.VM:VM/src/lmem.cpp:127:size_class_config`
//! Source: `VM/src/lmem.cpp:127-176` (hand-fixed: the constexpr constructor
//! was ported as an instance method nobody called, the `kSizeClassConfig`
//! static referenced by `sizeclass!` never existed, and kSizeClasses said 32
//! where the C++ says LUA_SIZECLASSES = 40 — the table is now built at
//! compile time by a const fn, faithful to the C++ constructor)

#[allow(non_upper_case_globals)]
pub(crate) const kSizeClasses: usize = 40; // LUA_SIZECLASSES
#[allow(non_upper_case_globals)]
pub(crate) const kMaxSmallSize: usize = 1024;

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug)]
pub struct SizeClassConfig {
    pub sizeOfClass: [core::ffi::c_int; kSizeClasses],
    pub classForSize: [core::ffi::c_char; kMaxSmallSize + 1],
    pub classCount: core::ffi::c_int,
}

const fn build_size_class_config() -> SizeClassConfig {
    let mut size_of_class = [0i32; kSizeClasses];
    let mut class_for_size = [-1i8 as core::ffi::c_char; kMaxSmallSize + 1];
    let mut class_count = 0usize;

    // we use a progressive size class scheme:
    // - all size classes are aligned by 8b to satisfy pointer alignment requirements
    // - we first allocate sizes classes in multiples of 8
    // - after the first cutoff we allocate size classes in multiples of 16
    // - after the second cutoff we allocate size classes in multiples of 32
    // - after the third cutoff we allocate size classes in multiples of 64
    // this balances internal fragmentation vs external fragmentation
    let mut size = 8;
    while size < 64 {
        size_of_class[class_count] = size;
        class_count += 1;
        size += 8;
    }
    let mut size = 64;
    while size < 256 {
        size_of_class[class_count] = size;
        class_count += 1;
        size += 16;
    }
    let mut size = 256;
    while size < 512 {
        size_of_class[class_count] = size;
        class_count += 1;
        size += 32;
    }
    let mut size = 512;
    while size <= 1024 {
        size_of_class[class_count] = size;
        class_count += 1;
        size += 64;
    }

    assert!(class_count <= kSizeClasses);

    // fill the lookup table for all classes
    let mut klass = 0usize;
    while klass < class_count {
        class_for_size[size_of_class[klass] as usize] = klass as core::ffi::c_char;
        klass += 1;
    }

    // fill the gaps in lookup table
    let mut size = kMaxSmallSize as i32 - 1;
    while size >= 0 {
        if (class_for_size[size as usize] as i8) < 0 {
            class_for_size[size as usize] = class_for_size[size as usize + 1];
        }
        size -= 1;
    }

    SizeClassConfig {
        sizeOfClass: size_of_class,
        classForSize: class_for_size,
        classCount: class_count as core::ffi::c_int,
    }
}

#[allow(non_upper_case_globals)]
pub static kSizeClassConfig: SizeClassConfig = build_size_class_config();
