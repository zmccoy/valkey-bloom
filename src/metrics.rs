use lazy_static::lazy_static;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use valkey_module::{InfoContext, ValkeyResult};

lazy_static! {
    pub static ref BLOOM_NUM_OBJECTS: AtomicU64 = AtomicU64::new(0);
    pub static ref BLOOM_OBJECT_TOTAL_MEMORY_BYTES: AtomicUsize = AtomicUsize::new(0);
    pub static ref BLOOM_NUM_FILTERS_ACROSS_OBJECTS: AtomicU64 = AtomicU64::new(0);
    pub static ref BLOOM_NUM_ITEMS_ACROSS_OBJECTS: AtomicU64 = AtomicU64::new(0);
    pub static ref BLOOM_CAPACITY_ACROSS_OBJECTS: AtomicU64 = AtomicU64::new(0);
    pub static ref BLOOM_DEFRAG_HITS: AtomicU64 = AtomicU64::new(0);
    pub static ref BLOOM_DEFRAG_MISSES: AtomicU64 = AtomicU64::new(0);

    //Count-min-sketch metrics
    pub static ref CMS_NUM_OBJECTS: AtomicU64 = AtomicU64::new(0);
    pub static ref CMS_OBJECT_TOTAL_MEMORY_BYTES: AtomicUsize = AtomicUsize::new(0);
    pub static ref CMS_NUM_FILTERS_ACROSS_OBJECTS: AtomicU64 = AtomicU64::new(0);
    pub static ref CMS_NUM_ITEMS_ACROSS_OBJECTS: AtomicU64 = AtomicU64::new(0);
    pub static ref CMS_CAPACITY_ACROSS_OBJECTS: AtomicU64 = AtomicU64::new(0);
    pub static ref CMS_DEFRAG_HITS: AtomicU64 = AtomicU64::new(0);
    pub static ref CMS_DEFRAG_MISSES: AtomicU64 = AtomicU64::new(0);

}

pub fn bloom_info_handler(ctx: &InfoContext) -> ValkeyResult<()> {
    ctx.builder()
        .add_section("bloom_core_metrics")
        .field(
            "bloom_total_memory_bytes",
            BLOOM_OBJECT_TOTAL_MEMORY_BYTES
                .load(Ordering::Relaxed)
                .to_string(),
        )?
        .field(
            "bloom_num_objects",
            BLOOM_NUM_OBJECTS.load(Ordering::Relaxed).to_string(),
        )?
        .field(
            "bloom_num_filters_across_objects",
            BLOOM_NUM_FILTERS_ACROSS_OBJECTS
                .load(Ordering::Relaxed)
                .to_string(),
        )?
        .field(
            "bloom_num_items_across_objects",
            BLOOM_NUM_ITEMS_ACROSS_OBJECTS
                .load(Ordering::Relaxed)
                .to_string(),
        )?
        .field(
            "bloom_capacity_across_objects",
            BLOOM_CAPACITY_ACROSS_OBJECTS
                .load(Ordering::Relaxed)
                .to_string(),
        )?
        .build_section()?
        .add_section("bloom_defrag_metrics")
        .field(
            "bloom_defrag_hits",
            BLOOM_DEFRAG_HITS.load(Ordering::Relaxed).to_string(),
        )?
        .field(
            "bloom_defrag_misses",
            BLOOM_DEFRAG_MISSES.load(Ordering::Relaxed).to_string(),
        )?
        .build_section()?
        .build_info()?;

    Ok(())
}

pub fn cms_info_handler(ctx: &InfoContext) -> ValkeyResult<()> {
    ctx.builder()
        .add_section("cms_core_metrics")
        .field(
            "cms_total_memory_bytes",
            CMS_OBJECT_TOTAL_MEMORY_BYTES
                .load(Ordering::Relaxed)
                .to_string(),
        )?
        .field(
            "cms_num_objects",
            CMS_NUM_OBJECTS.load(Ordering::Relaxed).to_string(),
        )?
        .field(
            "cms_num_filters_across_objects",
            CMS_NUM_FILTERS_ACROSS_OBJECTS
                .load(Ordering::Relaxed)
                .to_string(),
        )?
        .field(
            "cms_num_items_across_objects",
            CMS_NUM_ITEMS_ACROSS_OBJECTS
                .load(Ordering::Relaxed)
                .to_string(),
        )?
        .field(
            "cms_capacity_across_objects",
            CMS_CAPACITY_ACROSS_OBJECTS
                .load(Ordering::Relaxed)
                .to_string(),
        )?
        .build_section()?
        .add_section("cms_defrag_metrics")
        .field(
            "cms_defrag_hits",
            CMS_DEFRAG_HITS.load(Ordering::Relaxed).to_string(),
        )?
        .field(
            "cms_defrag_misses",
            CMS_DEFRAG_MISSES.load(Ordering::Relaxed).to_string(),
        )?
        .build_section()?
        .build_info()?;

    Ok(())
}
