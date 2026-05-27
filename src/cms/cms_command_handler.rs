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

    // Maximum allowable error rate.  Epsilon
    // For an epsilon of 1% and a estimated count of 1000 the actual could be between 990 and 1010
    let error_rate = match args[2].to_string_lossy().parse::<f64>() {
        Ok(e) if e > 0.0 && e < 1.0 => e,
        Ok(_) => return Err(ValkeyError::Str(utils::ERROR_RATE_RANGE)),
        Err(_) => return Err(ValkeyError::Str(utils::BAD_ERROR_RATE)),
    };

    //False positive rate. Delta
    // A delta of 1% means the count will be outside of the epsilon range 1% of the time.
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
    let args_count = args.len();
    if args_count < 4 {
        return Err(ValkeyError::WrongArity);
    }

    let key = &args[1];

    let args_left = args_count - 2;
    let is_even = args_left % 2 == 0;
    if !is_even {
        return Err(ValkeyError::WrongArity);
    }

    let mut i = 2;
    let mut pairs: Vec<(&ValkeyString, &ValkeyString)> = Vec::new();
    while i < args_count {
        let k = &args[i];
        let v = &args[i + 1];
        pairs.push((k, v));
        i += 2
    }

    let filter_key = ctx.open_key_writable(key);
    let value = match filter_key.get_value::<CMSObject>(&CMS_TYPE) {
        Ok(v) => v,
        Err(_) => return Err(ValkeyError::WrongType),
    };

    let mut results = Vec::new();
    match value {
        None => Err(ValkeyError::nonexistent_key()),
        Some(v) => {
            for (item, increment) in pairs {
                let key = &item.to_string_lossy();
                let parsed_value = &increment.to_string_lossy().parse::<u64>();
                let value = match parsed_value {
                    Ok(v) => v,
                    Err(_) => return Err(ValkeyError::WrongType),
                };
                let count = v.incrementy_by(key, value.to_owned());
                results.push(ValkeyValue::Integer(count as i64)); //Yet again a conversion issue to come back to.
            }
            //TODO: Replicate and notify
            Ok(ValkeyValue::Array(results))
        }
    }
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
            let estimates: Vec<ValkeyValue> = args[2..]
                .iter()
                .map(|item| {
                    let estimate = v.estimate_frequency(item.to_string_lossy().as_str());
                    ValkeyValue::Integer(estimate as i64)
                })
                .collect();
            Ok(ValkeyValue::Array(estimates))
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

    match cms {
        Some(cms) => {
            //TODO: Rework types on i64 vs u64
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
