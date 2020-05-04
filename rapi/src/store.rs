use actix_web::{get, web, HttpRequest, HttpResponse, Result};

use crate::CONFIG;
use azure_sdk_core::prelude::*;
use azure_sdk_storage_blob::prelude::*;
use azure_sdk_storage_core::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;

fn tok(req: HttpRequest) -> String {
    req.headers()
        .get("Authorization")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

#[get("/")]
async fn list(req: HttpRequest) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(
        client::with_bearer_token(&CONFIG.azure_storage_account, tok(req))
            .list_containers()
            .finalize()
            .await
            .unwrap()
            .incomplete_vector
            .iter()
            .map(|c| c.name.clone())
            .collect::<Vec<_>>(),
    ))
}

#[derive(Deserialize, Serialize, Clone)]
struct Manifest {
    dimensions: Vec<Vec<i32>>,
    samples: i32,
}

async fn manifest(req: HttpRequest, container_name: &str) -> Manifest {
    let client = client::with_bearer_token(&CONFIG.azure_storage_account, tok(req));
    let resp = client
        .get_blob()
        .with_container_name(container_name)
        .with_blob_name("manifest.json")
        .finalize()
        .await
        .unwrap()
        .data;
    serde_json::from_slice(&resp).unwrap()
}

#[get("/{container_name}/slice")]
async fn dimensions(container_name: web::Path<String>, req: HttpRequest) -> Result<HttpResponse> {
    let m = manifest(req, container_name.as_str()).await;
    let mut v = Vec::new();
    for i in 0..m.dimensions.len() {
        v.push(i);
    }
    Ok(HttpResponse::Ok().json(v))
}

#[get("/{container_name}/slice/{dim}")]
async fn lines(path: web::Path<(String, usize)>, req: HttpRequest) -> Result<HttpResponse> {
    let m = manifest(req, path.0.as_str()).await;
    Ok(HttpResponse::Ok().json(m.dimensions[path.1].clone()))
}
