use jsonwebtoken::DecodingKey;
use std::collections::{hash_map::RandomState, HashMap};

#[derive(Debug)]
pub struct AppState<'a> {
    pub jwks: HashMap<String, DecodingKey<'a>, RandomState>,
    pub validation: jsonwebtoken::Validation,
}
