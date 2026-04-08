mod lazy_writer_and_cache;
pub use lazy_writer_and_cache::LazyWriterAndCache;

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct UserId(pub u64);
