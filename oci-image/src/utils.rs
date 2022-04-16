use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Generate a new unique identifier from a string entry
pub fn to_uid(param: &str) -> String {
    let mut hasher = DefaultHasher::new();
    param.hash(&mut hasher);
    hasher.finish().to_string()
}
