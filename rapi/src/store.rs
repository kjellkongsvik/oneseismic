use crate::oneseismic;
use crate::slice_model;
use crate::state::AppState;
use actix_http::ResponseBuilder;
use actix_web::{error, http::header, http::StatusCode, web, HttpResponse, Result};
use log::info;
use log::trace;

pub fn service(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/{account}").route(web::get().to(list)));
}

async fn list<'a>(p: web::Path<String>, state: web::Data<AppState<'a>>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json("ddd"))
}

mod tests {
    use azure_sdk_core::prelude::*;
    use azure_sdk_storage_blob::prelude::*;
    use azure_sdk_storage_core::prelude::*;
    use url::Url;

    #[tokio::test]
    #[ignore]
    async fn fetch_ok() {
        // this is how you use the emulator
        let blob_storage_url = "http://127.0.0.1:10000";
        let table_storage_url = "http://127.0.0.1:10002";
        let client = client::with_emulator(
            &Url::parse(blob_storage_url).unwrap(),
            &Url::parse(table_storage_url).unwrap(),
        );

        let res = client
            .create_container()
            .with_container_name("emulcont")
            .with_public_access(PublicAccess::None)
            .finalize()
            .await
            .unwrap();
        println!("{:?}", res);
        if let Ok(conts) = client.list_containers().finalize().await {
            for c in conts.incomplete_vector.iter() {
                println!("{:?}", c.name);
                client
                    .delete_container()
                    .with_container_name(&c.name)
                    .finalize()
                    .await
                    .unwrap();
            }
        }
    }
}
