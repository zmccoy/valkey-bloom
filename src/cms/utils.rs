use count_min_sketch::{CountMinSketch64, CountMinSketch8};

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
    pub width: u64,
    pub depth: u64,
    pub total: u64,
    cms: CMS,
}

impl CMSObject {
    pub fn new_by_dimension(width: u64, depth: u64) -> Result<CMSObject, CMSError> {
        if width == 0 {
            return Err(CMSError::InvalidWidth);
        }

        if depth == 0 {
            return Err(CMSError::InvalidDepth);
        }

        let cms = CMS::new_by_probability(10000, 10.8, 1.0)?; //TODO
        let obj = CMSObject {
            width,
            depth,
            total: 0,
            cms
        };
//        obj.cms_object_incr_metrics_on_new_create();
        Ok(obj)
    }

    pub fn new_by_probability(error: f64, probability: f64) -> Result<CMSObject, CMSError> {
        if error <= 0.0 || error >= 1.0 {
            return Err(CMSError::InvalidErrorRate);
        }
        if probability <= 0.0 || probability >= 1.0 {
            return Err(CMSError::InvalidProbability);
        }

        // width = ceil(e / error)
        let width = (std::f64::consts::E / error).ceil() as u64;
        // depth = ceil(ln(1 / probability))
        let depth = (1.0_f64 / probability).ln().ceil() as u64;

        let cms = CMS::new_by_probability(width, error, probability)?;
        let obj = CMSObject {
            width,
            depth,
            total: 0,
            cms
        };
//        obj.cms_object_incr_metrics_on_new_create();
        Ok(obj)
    }

    pub fn estimate_frequency(&self, k: &str) -> u64 {
        self.cms.estimate_frequency(k)
    }

    pub fn incrementy_by(& mut self, item: &str, increment: u64) -> u64 {
        self.cms.increment_by(item, increment)
    }


}

//TODO: Implement typeclass for CMS usage
struct CMS {
    sketch: CountMinSketch64<String>,
}

impl CMS {

    pub fn new_by_probability(width: u64, error: f64, probability: f64) -> Result<CMS, CMSError> {

        let tolerance = error;
        let confidence = 1.0 - probability;
        let cms = CountMinSketch64::new(width as usize, confidence, tolerance).map_err(|_| CMSError::InvalidWidth)?;
        Ok(
            CMS {
                sketch: cms,
            }
        )
    }

    pub fn estimate_frequency(&self, k: &str) -> u64 {
        self.sketch.estimate(k)
    }

    //TODO Come back to this implementation, where things get composed.
    pub fn increment_by(& mut self, item: &str, increment: u64) -> u64 {
        self.sketch.add(item, increment);
        self.estimate_frequency(item)
    }
}
