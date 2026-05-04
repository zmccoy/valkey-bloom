use valkey_module::{Context, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue, VALKEY_OK};

use crate::cms::data_type::CMS_TYPE;
use crate::cms::utils::{self, CMSObject};

/// Function that implements logic to handle the CMS.INITBYDIM command.
pub fn cms_initialize_by_dimensions(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let args_count = args.len();
    if args_count != 4 {
        return Err(valkey_module::ValkeyError::WrongArity);
    }

    let key = &args[1];

    let width = match args[2].to_string_lossy().parse::<u64>() {
        Ok(w) if w > 0 => w,
        _ => return Err(ValkeyError::Str(utils::BAD_WIDTH)),
    };

    let depth = match args[3].to_string_lossy().parse::<u64>() {
        Ok(d) if d > 0 => d,
        _ => return Err(ValkeyError::Str(utils::BAD_DEPTH)),
    };

    let filter_key = ctx.open_key_writable(key);
    let cms = match filter_key.get_value::<CMSObject>(&CMS_TYPE) {
        Ok(v) => v,
        Err(_) => return Err(ValkeyError::WrongType),
    };

    match cms {
        Some(_) => Err(ValkeyError::Str(utils::ITEM_EXISTS)),
        None => {
            let cms = match utils::CMSObject::new_by_dimension(width, depth) {
                Ok(v) => v,
                Err(err) => return Err(ValkeyError::Str(err.as_str())),
            };

           //TODO: Replication Args need done still

            match filter_key.set_value(&CMS_TYPE, cms) {
                Ok(()) => {
                    //replicate_and_notify_events(ctx, filter_name, false, true, replicate_args);
                    VALKEY_OK
                }
                Err(_) => Err(ValkeyError::Str(utils::ERROR)),
            }
        }
    }
}

/// Function that implements logic to handle the CMS.INITBYPROB command.
pub fn cms_initialize_by_probability(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let args_count = args.len();
    if args_count != 4 {
        return Err(ValkeyError::WrongArity);
    }

    let key = &args[1];

    let error_rate = match args[2].to_string_lossy().parse::<f64>() {
        Ok(e) if e > 0.0 && e < 1.0 => e,
        Ok(_) => return Err(ValkeyError::Str(utils::ERROR_RATE_RANGE)),
        Err(_) => return Err(ValkeyError::Str(utils::BAD_ERROR_RATE)),
    };

    let probability = match args[3].to_string_lossy().parse::<f64>() {
        Ok(p) if p > 0.0 && p < 1.0 => p,
        Ok(_) => return Err(ValkeyError::Str(utils::PROBABILITY_RANGE)),
        Err(_) => return Err(ValkeyError::Str(utils::BAD_PROBABILITY)),
    };

    let filter_key = ctx.open_key_writable(key);
    let cms = match filter_key.get_value::<CMSObject>(&CMS_TYPE) {
        Ok(v) => v,
        Err(_) => return Err(ValkeyError::WrongType),
    };

    match cms {
        Some(_) => Err(ValkeyError::Str(utils::ITEM_EXISTS)),
        None => {
            let cms = match utils::CMSObject::new_by_probability(error_rate, probability) {
                Ok(v) => v,
                Err(err) => return Err(ValkeyError::Str(err.as_str())),
            };

           //TODO: Replication Args need done still
            match filter_key.set_value(&CMS_TYPE, cms) {
                Ok(()) => {
                    //replicate_and_notify_events(ctx, filter_name, false, true, replicate_args);
                    VALKEY_OK
                }
                Err(_) => Err(ValkeyError::Str(utils::ERROR)),
            }

        }
    }
}


/// Function that implements logic to handle the CMS.INCRBY command.
pub fn cms_increment_by(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    todo!();
}

/// Function that implements logic to handle the CMS.QUERY command.
pub fn cms_query(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let argc = args.len();
    if argc < 3 {
        return Err(ValkeyError::WrongArity);
    }

    let key_name = &args[1];

    let existing_key = ctx.open_key(key_name);
    let cms = match existing_key.get_value::<CMSObject>(&CMS_TYPE) {
        Ok(v) => v,
        Err(_) => return Err(ValkeyError::WrongType),
    };

    match cms {
        None => Err(ValkeyError::Str("No CMS Exists")),
        Some(v) => {
            let key = &args[2]; //TODO: This needs to loop through all the rest of the array of args.
            let estimate = v.estimate_frequency(key.to_string_lossy().as_str());
            Ok(ValkeyValue::Integer(estimate as i64)) //TODO: What's the correct conversion for all of these
        }
    }
}

/// Function that implements logic to handle the CMS.INFO command.
pub fn cms_info(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let args_count = args.len();
    if args_count != 2 {
        return Err(valkey_module::ValkeyError::WrongArity);
    }

    let key = &args[1];
    let key_existing = ctx.open_key(key);
    let cms = match key_existing.get_value::<CMSObject>(&CMS_TYPE) {
        Ok(v) => v,
        Err(_) => return Err(ValkeyError::WrongType),
    };

    //TODO: Utilize CMSObject
    match cms {
        Some(cms) => { //TODO: Rework types on i64 vs u64
            let result = vec![
                ValkeyValue::SimpleStringStatic("Width"),
                ValkeyValue::Integer(cms.width as i64),
                ValkeyValue::SimpleStringStatic("Depth"),
                ValkeyValue::Integer(cms.depth as i64),
                ValkeyValue::SimpleStringStatic("Count"),
                ValkeyValue::Integer(cms.total as i64),
            ];
            Ok(ValkeyValue::Array(result))
        }
        None => Err(ValkeyError::Str(utils::NOT_FOUND)),
    }
}

/// Function that implements logic to handle the CMS.MERGE command.
pub fn cms_merge(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    todo!();
}
