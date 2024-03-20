use crate::blocks::EmbeddedBlock;
use crate::embeddings::encoder::NearestVectors;
use rusqlite::Connection;

pub enum VectorStore {
    SQLiteStore(Connection),
}

use crate::embeddings::encoder::Embeddings;
use crate::vector_store::sqlite::{ProjectInfo, SQLite};

static DB_PATH: &str = "db/blockoli.sqlite";

impl VectorStore {
    pub fn init_sqlite() -> VectorStore {
        let connection = Connection::open(DB_PATH).unwrap();
        VectorStore::SQLiteStore(connection)
    }

    pub async fn create_project(&self, project_name: &str) {
        match self {
            VectorStore::SQLiteStore(conn) => {
                SQLite::create_table(conn, &project_name).unwrap();
            }
        }
    }

    pub async fn delete_project(&self, project_name: &str) {
        match self {
            VectorStore::SQLiteStore(conn) => {
                SQLite::delete_project(conn, project_name).unwrap();
            }
        }
    }

    pub async fn does_project_exist(&self, project_name: &str) -> bool {
        match self {
            VectorStore::SQLiteStore(conn) => {
                SQLite::does_project_exist(conn, project_name).unwrap()
            }
        }
    }

    pub async fn get_project_info(&self, project_name: &str) -> Option<ProjectInfo> {
        match self {
            VectorStore::SQLiteStore(conn) => SQLite::get_project_info(conn, project_name).unwrap(),
        }
    }

    pub async fn insert_blocks(&mut self, project_name: &str, blocks: Vec<EmbeddedBlock>) {

        match self {
            VectorStore::SQLiteStore(conn) => {
                SQLite::insert_blocks(conn, project_name, blocks).unwrap();
            }
        }
    }

    pub async fn search(&self, project_name: &str, search_code: String) -> NearestVectors {
        match self {
            VectorStore::SQLiteStore(conn) => {
                let code_vectors = SQLite::get_code_vectors(conn, project_name).unwrap();

                Embeddings::search(code_vectors, search_code, 5).unwrap()
            }
        }
    }

    pub async fn get_all_function_blocks(&self, project_name: &str) -> Vec<asterisk::block::Block> {
        match self {
            VectorStore::SQLiteStore(conn) => {
                SQLite::get_all_function_blocks(conn, project_name).unwrap()
            }
        }
    }

    pub async fn search_from_function_blocks(
        &self,
        project_name: &str,
        search_code: String,
    ) -> Vec<asterisk::block::Block> {
        match self {
            VectorStore::SQLiteStore(conn) => {
                SQLite::search_from_function_blocks(conn, project_name, &search_code).unwrap()
            }
        }
    }

    pub async fn search_by_function_name(
        &self,
        project_name: &str,
        function_name: String,
    ) -> Vec<asterisk::block::Block> {
        match self {
            VectorStore::SQLiteStore(conn) => {
                SQLite::search_by_function_name(conn, project_name, &function_name).unwrap()
            }
        }
    }
}
