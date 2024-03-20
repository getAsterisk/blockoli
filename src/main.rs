use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use parking_lot::Mutex;
use std::sync::Arc;

use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod blocks;
mod embeddings;
mod routes;
mod vector_store;

use routes::*;
use vector_store::vector_store::VectorStore;

pub struct AppState {
    pub vector_store: Arc<Mutex<VectorStore>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let port = args
        .get(1)
        .expect("Error: No port provided\nUsage: blockoli <sqlite/qdrant> <port>")
        .to_owned();

    let vector_store = Arc::new(Mutex::new(VectorStore::init_sqlite()));

    let url = "127.0.0.1";
    println!("blockoli server starting on {}. Port: {}", url, port);

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(AppState {
                vector_store: vector_store.clone(),
            }))
            .route("/project", web::post().to(create_project))
            .route("/project/{project_name}", web::get().to(project_info))
            .route("/project/{project_name}", web::delete().to(delete_project))
            .route("/project/generate", web::post().to(generate_embeddings))
            .route("/search/{project_name}", web::post().to(search_embeddings))
            .route(
                "/get_blocks/{project_name}",
                web::post().to(get_all_function_blocks),
            )
            .route(
                "/search_blocks/{project_name}",
                web::post().to(search_function_blocks),
            )
            .route(
                "/search_by_function/{project_name}",
                web::post().to(search_by_function_name),
            )
    })
    .bind(format!("{}:{}", url, port))?
    .run()
    .await
}
