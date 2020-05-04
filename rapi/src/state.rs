use crate::multiplexer;
use jsonwebtoken::DecodingKey;
use std::collections::{hash_map::RandomState, HashMap};

#[derive(Debug)]
pub struct AppState<'a> {
    pub sender: tokio::sync::mpsc::Sender<multiplexer::Job>,
    pub jwks: HashMap<String, DecodingKey<'a>, RandomState>,
    pub validation: jsonwebtoken::Validation,
    pub root: String,
}
