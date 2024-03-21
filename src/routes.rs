use crate::embeddings::encoder::Embeddings;
use crate::AppState;
use actix_web::http::header::CROSS_ORIGIN_EMBEDDER_POLICY;
use actix_web::web;
use actix_web::Responder;
use actix_web::{HttpRequest, HttpResponse};
use jwalk::WalkDir;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::fs;

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

/// Creates a new project.
///
/// Expects a JSON body with the following fields:
/// - `project_name`: The name of the project to create.
/// - `project_path`: The filesystem path to the project.
///
/// # Returns
///
/// - `200 OK` if the project was created successfully.
/// - `400 Bad Request` if the request body is missing required fields.
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

/// Deletes a project.
///
/// # Arguments
///
/// * `info` - A `web::Path<String>` containing the name of the project to delete.
///
/// # Returns
///
/// - `200 OK` with a JSON body containing a success message if the project was deleted.
/// - `404 Not Found` if no project with the given name exists.
pub async fn delete_project(
    info: web::Path<String>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let project_name = info.to_owned();

    app_state
        .vector_store
        .lock()
        .delete_project(&project_name)
        .await;

    HttpResponse::Ok().content_type("application/json").body(
        serde_json::to_string_pretty(&ErrorResponse {
            message: format!("Deleted project {}", project_name),
        })
        .unwrap(),
    )
}

pub async fn project_info(
    info: web::Path<String>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let project_name = info.to_owned();

    let project_info = app_state
        .vector_store
        .lock()
        .get_project_info(&project_name)
        .await;

    match project_info {
        Some(info) => HttpResponse::Ok()
            .content_type("application/json")
            .body(serde_json::to_string_pretty(&info).unwrap()),
        None => HttpResponse::NotFound()
            .content_type("application/json")
            .body(
                serde_json::to_string_pretty(&ErrorResponse {
                    message: format!("Project {} not found", project_name),
                })
                .unwrap(),
            ),
    }
}

/// Generates vector embeddings for the code files in a project and inserts them into the vector store.
///
/// Expects a JSON body with the following fields:
/// - `project_name`: The name of the project to generate embeddings for. Must already exist in the vector store.
/// - `project_path`: The filesystem path to the project's code files.
///
/// # Returns
///
/// - `200 OK` with a JSON body containing the project name and path and a success message.
/// - `404 Not Found` if no project with the given name exists in the vector store.  
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
            .body(
                serde_json::to_string_pretty(&ErrorResponse {
                    message: format!("Project {} not found", project_name),
                })
                .unwrap(),
            );
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
            vectors: code_vectors[i].point.to_vec(),
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

/// Searches a project for code blocks matching the given code query, using vector embeddings.
///
/// # Arguments
///
/// * `info` - A `web::Path<String>` containing the name of the project to search in. Must exist in the vector store.
/// * `data` - The code to search for matches to, as a raw request body.
///  
/// # Returns
///
/// - `200 OK` with a JSON body containing the most similar code block and a list of the closest matching blocks.
/// - `404 Not Found` if no project with the given name exists in the vector store.
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
            .body(
                serde_json::to_string_pretty(&ErrorResponse {
                    message: format!("Project {} not found", project_name),
                })
                .unwrap(),
            );
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

/// Retrieves all code blocks from a project that are non-empty functions.
///  
/// # Arguments
///
/// * `info` - A `web::Path<String>` containing the name of the project to retrieve blocks from. Must exist in the vector store.
/// * `_req` - The HTTP request (unused).
///
/// # Returns
///
/// - `200 OK` with a JSON body containing the retrieved code blocks.
/// - `404 Not Found` if no project with the given name exists in the vector store.
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
            .body(
                serde_json::to_string_pretty(&ErrorResponse {
                    message: format!("Project {} not found", project_name),
                })
                .unwrap(),
            );
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

/// Searches for code blocks matching a query in a project, filtering for non-empty functions.
///
/// # Arguments
///
/// * `info` - A `web::Path<String>` containing the name of the project to search in. Must exist in the vector store.
/// * `_req` - The HTTP request (unused).  
/// * `data` - The code to search for matches to, as a raw request body.
///
/// # Returns
///
/// - `200 OK` with a JSON body containing the code blocks matching the query.
/// - `404 Not Found` if no project with the given name exists in the vector store.
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
            .body(
                serde_json::to_string_pretty(&ErrorResponse {
                    message: format!("Project {} not found", project_name),
                })
                .unwrap(),
            );
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

/// Searches for code blocks with a specific function name in a project.
///
/// # Arguments
///
/// * `info` - A `web::Path<String>` containing the name of the project to search in. Must exist in the vector store.
/// * `_req` - The HTTP request (unused).
/// * `data` - The function name to search for, as a raw request body.  
///
/// # Returns
///
/// - `200 OK` with a JSON body containing the code blocks with the given function name.
/// - `404 Not Found` if no project with the given name exists in the vector store.
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
            .body(
                serde_json::to_string_pretty(&ErrorResponse {
                    message: format!("Project {} not found", project_name),
                })
                .unwrap(),
            );
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
