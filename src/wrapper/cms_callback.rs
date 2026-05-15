use crate::bloom;
use crate::bloom::data_type::ValkeyDataType;
use crate::bloom::utils::CMSObject;

// Note: methods in this mod are for the cms module data type callbacks.
// The reason they are unsafe is because the callback methods are expected to be
// "unsafe extern C" based on the Rust module API definition

pub unsafe extern "C" fn cms_rdb_save(rdb: *mut raw::RedisModuleIO, value: *mut c_void) {
    let v = &*value.cast::<CMSObject>();
    todo!()
}


pub unsafe extern "C" fn cms_rdb_load(rdb: *mut raw::RedisModuleIO, encver: c_int,) -> *mut c_void {
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

pub unsafe extern "C" fn bloom_aux_load(rdb: *mut raw::RedisModuleIO, _encver: c_int, _when: c_int) -> c_int {
   // cms::data_type::cms_rdb_aux_load(rdb)
   todo!()
}
