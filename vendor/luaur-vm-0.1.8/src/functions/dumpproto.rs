use crate::functions::dumpref::dumpref;
use crate::functions::dumprefs::dumprefs;
use crate::functions::dumpstringdata::dumpstringdata;
use crate::macros::obj_2_gco::obj2gco;
use crate::type_aliases::instruction::Instruction;
use crate::type_aliases::loc_var::LocVar;
use crate::type_aliases::proto::Proto;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub(crate) unsafe fn dumpproto(f: *mut core::ffi::c_void, p: *mut Proto) {
    let p = &*p;

    let size = core::mem::size_of::<Proto>()
        + core::mem::size_of::<Instruction>() * p.sizecode as usize
        + core::mem::size_of::<*mut Proto>() * p.sizep as usize
        + core::mem::size_of::<TValue>() * p.sizek as usize
        + p.sizelineinfo as usize
        + core::mem::size_of::<LocVar>() * p.sizelocvars as usize
        + core::mem::size_of::<*mut crate::records::t_string::TString>() * p.sizeupvalues as usize;

    extern "C" {
        fn fprintf(
            stream: *mut core::ffi::c_void,
            format: *const core::ffi::c_char,
            ...
        ) -> core::ffi::c_int;
    }

    fprintf(
        f,
        b"{\"type\":\"proto\",\"cat\":%d,\"size\":%d\0".as_ptr() as *const core::ffi::c_char,
        p.hdr.memcat as core::ffi::c_int,
        size as core::ffi::c_int,
    );

    if !p.source.is_null() {
        fprintf(f, b",\"source\":\"\0".as_ptr() as *const core::ffi::c_char);
        dumpstringdata(f, (*p.source).data.as_ptr(), (*p.source).len as usize);
        fprintf(
            f,
            b"\",\"line\":%d\0".as_ptr() as *const core::ffi::c_char,
            if !p.abslineinfo.is_null() {
                *p.abslineinfo
            } else {
                0
            },
        );
    }

    if p.sizek > 0 {
        fprintf(
            f,
            b",\"constants\":[\0".as_ptr() as *const core::ffi::c_char,
        );
        dumprefs(f, p.k, p.sizek as usize);
        fprintf(f, b"]\0".as_ptr() as *const core::ffi::c_char);
    }

    if p.sizep > 0 {
        fprintf(f, b",\"protos\":[\0".as_ptr() as *const core::ffi::c_char);
        for i in 0..p.sizep as usize {
            if i != 0 {
                extern "C" {
                    fn fputc(
                        c: core::ffi::c_int,
                        stream: *mut core::ffi::c_void,
                    ) -> core::ffi::c_int;
                }
                fputc(',' as core::ffi::c_int, f);
            }
            let proto_ptr = *p.p.add(i);
            dumpref(
                f,
                &mut (*proto_ptr).hdr as *mut crate::records::g_cheader::GCheader
                    as *mut crate::records::gc_object::GCObject,
            );
        }
        fprintf(f, b"]\0".as_ptr() as *const core::ffi::c_char);
    }

    fprintf(f, b"}\0".as_ptr() as *const core::ffi::c_char);
}
