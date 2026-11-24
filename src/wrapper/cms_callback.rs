use crate::cms::utils::CMSObject;
use std::os::raw::c_void;

/// # Safety
/// Frees a CMS object. Called by the server when a CMS key is deleted or flushed.
pub unsafe extern "C" fn cms_free(value: *mut c_void) {
    drop(Box::from_raw(value.cast::<CMSObject>()));
}
