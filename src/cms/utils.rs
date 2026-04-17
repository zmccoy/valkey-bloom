
/// Client Errors
pub const ERROR: &str = "ERROR";
pub const NOT_FOUND: &str = "ERR not found";
pub const ITEM_EXISTS: &str = "ERR item exists";
pub const BAD_WIDTH: &str = "ERR bad width";
pub const BAD_DEPTH: &str = "ERR bad depth";
pub const BAD_ERROR_RATE: &str = "ERR bad error rate";
pub const ERROR_RATE_RANGE: &str = "ERR error rate should be between 0 and 1";
pub const BAD_PROBABILITY: &str = "ERR bad probability";
pub const PROBABILITY_RANGE: &str = "ERR probability rate should be between 0 and 1";
pub const KEY_EXISTS: &str = "ERR Target key name already exists.";

#[derive(Debug, PartialEq)]
pub enum CMSError {
    InvalidWidth,
    InvalidDepth,
    InvalidErrorRate,
    InvalidProbability,
}

impl CMSError {
    pub fn as_str(&self) -> &'static str {
        match self {
            CMSError::InvalidWidth => BAD_WIDTH,
            CMSError::InvalidDepth => BAD_DEPTH,
            CMSError::InvalidErrorRate => ERROR_RATE_RANGE,
            CMSError::InvalidProbability => PROBABILITY_RANGE,
        }
    }
}

pub struct CMSObject {
    width: u64,
    depth: u64,
}

impl CMSObject {
    pub fn new_by_dimension(width: u64, depth: u64) -> Result<CMSObject, CMSError> {
        if width == 0 {
            return Err(CMSError::InvalidWidth);
        }

        if depth == 0 {
            return Err(CMSError::InvalidDepth);
        }

        //Create CMS

        Ok(todo!())
    }
}
