use crate::bloom::data_type::ValkeyDataType;
use flowstats::CountMinSketch;
use std::os::raw::{c_char, c_int, c_void};
use valkey_module::raw;

use crate::cms::utils::CMSObject;

// pub unsafe extern "C" fn cms_rdb_save(rdb: *mut raw::RedisModuleIO, value: *mut c_void) {
//     let v = &*value.cast::<CountMinSketch>();
//     let bits = v.into_bytes();
//         raw::RedisModule_SaveStringBuffer.unwrap()(
//             rdb,
//             bits.as_ptr().cast::<c_char>(),
//             bits.len(),
//         );
//     }

// /// # Safety
// pub unsafe extern "C" fn cms_rdb_load(
//     rdb: *mut raw::RedisModuleIO,
//     encver: c_int,
// ) -> *mut c_void {
//     if let Some(item) = <CMSObject as ValkeyDataType>::load_from_rdb(rdb, encver) {
//         let bb = Box::new(item);
//         Box::into_raw(bb).cast::<libc::c_void>()
//     } else {
//         logging::log_warning("Failed to restore cms object.");
//         null_mut()
//     }
// }
