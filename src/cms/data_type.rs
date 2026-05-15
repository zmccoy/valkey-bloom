use crate::cms::utils::CMSObject;
use valkey_module::digest::Digest;
use valkey_module::native_types::ValkeyType;
use valkey_module::raw;
use crate::wrapper::cms_callback;

const CMS_TYPE_ENCODING_VERSION: i32 = 1;

//Note this is mocked out for now.
pub static CMS_TYPE: ValkeyType = ValkeyType::new(
    "cntmnskch",
    CMS_TYPE_ENCODING_VERSION,
    raw::RedisModuleTypeMethods {
        version: raw::REDISMODULE_TYPE_METHOD_VERSION as u64,
        rdb_load: Some(cms_callback::cms_rdb_load),
        rdb_save: Some(cms_callback::cms_rdb_save),
        aof_rewrite: Some(cms_callback::cms_aof_rewrite),
        digest: None,

        mem_usage: None,
        free: None,

        aux_load: None,

        aux_save: None,
        aux_save2: None,
        aux_save_triggers: raw::Aux::Before as i32,

        free_effort: None,
        unlink: None,
        copy: None,
        defrag: None,

        mem_usage2: None,
        free_effort2: None,
        unlink2: None,
        copy2: None,
    },
);

pub trait ValkeyDataType {
    fn load_from_rdb(rdb: *mut raw::RedisModuleIO, encver: i32) -> Option<CMSObject>;
    fn debug_digest(&self, dig: Digest);
}

impl ValkeyDataType for CMSObject {
    fn load_from_rdb(rdb: *mut raw::RedisModuleIO, encver: i32) -> Option<CMSObject> {
        todo!()
    }

    fn debug_digest(&self, dig: Digest) {
        todo!()
    }
}
