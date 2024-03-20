use actix_web::http::header::CROSS_ORIGIN_EMBEDDER_POLICY;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use crate::AppState;
use actix_web::Responder;
use actix_web::web;
use actix_web::{HttpRequest, HttpResponse};
use jwalk::WalkDir;
use std::fs;
use crate::embeddings::encoder::Embeddings;

use crate::blocks::EmbeddedBlock;

#[derive(Deserialize)]
pub struct EmbeddingsPayload {
    project_name: String,
    project_path: String,
}

#[derive(Serialize)]
pub struct EmbeddingsResponse {
    project_name: String,
    project_path: String,
    message: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    message: String,
}

#[derive(Deserialize)]
pub struct CreateProject {
    project_name: String,
    project_path: String,
}

pub async fn create_project(
    data: web::Json<CreateProject>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let project_name = data.project_name.to_owned();

    app_state
        .vector_store
        .lock()
        .create_project(&project_name)
        .await;

    HttpResponse::Ok()
        .content_type("application/json")
        .status(StatusCode::OK)
        .finish()
}

pub async fn delete_project(info: web::Path<String>, app_state: web::Data<AppState>) -> impl Responder {
    let project_name = info.to_owned();

    app_state
        .vector_store
        .lock()
        .delete_project(&project_name)
        .await;

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string_pretty(&ErrorResponse {
            message: format!("Deleted project {}", project_name),
        })
        .unwrap())
}

pub async fn project_info(info: web::Path<String>, app_state: web::Data<AppState>) -> impl Responder {
    let project_name = info.to_owned();

    let project_info = app_state
        .vector_store
        .lock()
        .get_project_info(&project_name)
        .await;

    match project_info {
        Some(info) => {
            HttpResponse::Ok()
                .content_type("application/json")
                .body(serde_json::to_string_pretty(&info).unwrap())
        }
        None => {
            HttpResponse::NotFound()
                .content_type("application/json")
                .body(serde_json::to_string_pretty(&ErrorResponse {
                    message: format!("Project {} not found", project_name),
                })
                .unwrap())
        }
    }
}

pub async fn generate_embeddings(
    data: web::Json<EmbeddingsPayload>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let project_name = data.project_name.to_owned();
    let project_path = data.project_path.to_owned();

    // check if project exists
    let project_info = app_state
        .vector_store
        .lock()
        .does_project_exist(&project_name)
        .await;

    if !project_info {
        return HttpResponse::NotFound()
            .content_type("application/json")
            .body(serde_json::to_string_pretty(&ErrorResponse {
                message: format!("Project {} not found", project_name),
            })
            .unwrap());
    }

    let toml_str = fs::read_to_string("../asterisk/asterisk.toml").expect("Unable to read file");
    let asterisk_config = asterisk::config::Config::from_toml(&toml_str).unwrap();

    let (blocks, _, _) = asterisk::indexer::index_directory(&asterisk_config, &project_path);

    let code_blocks: Vec<String> = blocks.iter().map(|block| block.content.clone()).collect();
    let code_vectors = Embeddings::generate_vector_set(code_blocks).unwrap();

    let mut embedded_blocks: Vec<EmbeddedBlock> = Vec::new();
    for (i, block) in blocks.iter().enumerate() {
        embedded_blocks.push(EmbeddedBlock {
            block: block.clone(),
            vectors: code_vectors[i].point.to_vec()
        });
    }

    app_state
        .vector_store
        .lock()
        .insert_blocks(&project_name, embedded_blocks.clone())
        .await;

    let response = EmbeddingsResponse {
        project_name: project_name.to_owned(),
        project_path: project_path.to_owned(),
        message: format!("Generated embeddings for {}", project_name),
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string_pretty(&response).unwrap())
}

pub async fn search_embeddings(
    info: web::Path<String>,
    data: web::Bytes,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let project_name = info.to_owned();

    // check if project exists
    let project_info = app_state
        .vector_store
        .lock()
        .does_project_exist(&project_name)
        .await;

    if !project_info {
        return HttpResponse::NotFound()
            .content_type("application/json")
            .body(serde_json::to_string_pretty(&ErrorResponse {
                message: format!("Project {} not found", project_name),
            })
            .unwrap());
    }

    let search_code = std::str::from_utf8(&data).unwrap().to_owned();

    let nearest_vectors = app_state
        .vector_store
        .lock()
        .search(&project_name, search_code)
        .await;

    let res_json = serde_json::to_string_pretty(&nearest_vectors).unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .body(res_json)
}

pub async fn get_all_function_blocks(
    info: web::Path<String>,
    _req: HttpRequest,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let project_name = info.to_owned();

    // check if project exists
    let project_info = app_state
        .vector_store
        .lock()
        .does_project_exist(&project_name)
        .await;

    if !project_info {
        return HttpResponse::NotFound()
            .content_type("application/json")
            .body(serde_json::to_string_pretty(&ErrorResponse {
                message: format!("Project {} not found", project_name),
            })
            .unwrap());
    }

    let function_blocks = app_state
        .vector_store
        .lock()
        .get_all_function_blocks(&project_name)
        .await;

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string_pretty(&function_blocks).unwrap())
}

pub async fn search_function_blocks(
    info: web::Path<String>,
    _req: HttpRequest,
    data: web::Bytes,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let project_name = info.to_owned();

    // check if project exists
    let project_info = app_state
        .vector_store
        .lock()
        .does_project_exist(&project_name)
        .await;

    if !project_info {
        return HttpResponse::NotFound()
            .content_type("application/json")
            .body(serde_json::to_string_pretty(&ErrorResponse {
                message: format!("Project {} not found", project_name),
            })
            .unwrap());
    }    

    let search_code = std::str::from_utf8(&data).unwrap().to_owned();

    let function_blocks = app_state
        .vector_store
        .lock()
        .search_from_function_blocks(&project_name, search_code)
        .await;

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string_pretty(&function_blocks).unwrap())
}

pub async fn search_by_function_name(
    info: web::Path<String>,
    _req: HttpRequest,
    data: web::Bytes,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let project_name = info.to_owned();

    // check if project exists
    let project_info = app_state
        .vector_store
        .lock()
        .does_project_exist(&project_name)
        .await;

    if !project_info {
        return HttpResponse::NotFound()
            .content_type("application/json")
            .body(serde_json::to_string_pretty(&ErrorResponse {
                message: format!("Project {} not found", project_name),
            })
            .unwrap());
    }    

    let function_name = std::str::from_utf8(&data).unwrap().to_owned();

    let function_blocks = app_state
        .vector_store
        .lock()
        .search_by_function_name(&project_name, function_name)
        .await;

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string_pretty(&function_blocks).unwrap())
}
