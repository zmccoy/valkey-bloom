use crate::cms::data_type::ValkeyDataType;
use crate::cms::utils::CMSObject;
use std::os::raw::{c_int, c_void};
use std::ptr::null_mut;
use valkey_module::digest::Digest;
use valkey_module::logging;
use valkey_module::raw;
use valkey_module::RedisModuleDefragCtx;
use valkey_module::RedisModuleString;

// Note: methods in this mod are for the cms module data type callbacks.
// The reason they are unsafe is because the callback methods are expected to be
// "unsafe extern C" based on the Rust module API definition

pub unsafe extern "C" fn cms_rdb_save(rdb: *mut raw::RedisModuleIO, value: *mut c_void) {
    let v = &*value.cast::<CMSObject>();
    todo!()
}

pub unsafe extern "C" fn cms_rdb_load(rdb: *mut raw::RedisModuleIO, encver: c_int) -> *mut c_void {
    if let Some(item) = <CMSObject as ValkeyDataType>::load_from_rdb(rdb, encver) {
        let bc = Box::new(item);
        Box::into_raw(bc).cast::<libc::c_void>()
    } else {
        logging::log_warning("Failed to restore count-min sketch object");
        null_mut()
    }
}

pub unsafe extern "C" fn cms_aof_rewrite(
    aof: *mut raw::RedisModuleIO,
    key: *mut raw::RedisModuleString,
    value: *mut c_void,
) {
    let sketch = &*value.cast::<CMSObject>();
    //Serialize the CMS Object
    // sketch.encode_object() needs to be implemented

    todo!()
}

pub unsafe extern "C" fn bloom_aux_load(
    rdb: *mut raw::RedisModuleIO,
    _encver: c_int,
    _when: c_int,
) -> c_int {
    // cms::data_type::cms_rdb_aux_load(rdb)
    todo!()
}

/// Free a cms object
pub unsafe extern "C" fn cms_free(value: *mut c_void) {
    drop(Box::from_raw(value.cast::<CMSObject>()));
}

/// Compute the memory usage for a cms object.
pub unsafe extern "C" fn cms_mem_usage(value: *const c_void) -> usize {
    let item = &*value.cast::<CMSObject>();
    item.cms_object_memory_usage() as usize
}

/// Raw handler for the COPY command.
pub unsafe extern "C" fn cms_copy(
    _from_key: *mut RedisModuleString,
    _to_key: *mut RedisModuleString,
    value: *const c_void,
) -> *mut c_void {
    let curr_item = &*value.cast::<CMSObject>();
    let new_item = CMSObject::create_copy_from(curr_item);
    let bb = Box::new(new_item);
    Box::into_raw(bb).cast::<libc::c_void>()
}

/// Raw handler for the cms digest callback.
pub unsafe extern "C" fn cms_digest(md: *mut raw::RedisModuleDigest, value: *mut c_void) {
    let dig = Digest::new(md);
    let val = &*(value.cast::<CMSObject>());
    val.debug_digest(dig);
}

pub unsafe extern "C" fn cms_aux_load(
    rdb: *mut raw::RedisModuleIO,
    _encver: c_int,
    _when: c_int,
) -> c_int {
    todo!()
}

pub unsafe extern "C" fn cms_free_effort(
    rdb: *mut raw::RedisModuleString,
    value: *const c_void,
) -> usize {
    let curr_item = &*value.cast::<CMSObject>();
    curr_item.free_effort()
}

pub unsafe extern "C" fn cms_defrag(
    defrag_ctx: *mut RedisModuleDefragCtx,
    _from_key: *mut RedisModuleString,
    value: *mut *mut c_void,
) -> i32 {
    todo!()
}
