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
    /// Initializes a new SQLite-backed vector store.
    ///
    /// # Returns
    ///
    /// A `VectorStore` enum with the `:SQLiteStore` variant containing the SQLite connection.
    pub fn init_sqlite() -> VectorStore {
        let connection = Connection::open(DB_PATH).unwrap();
        VectorStore::SQLiteStore(connection)
    }

    /// Creates a new project in the vector store.
    ///
    /// # Arguments
    ///
    /// * `self` - The `VectorStore` to create the project in.
    /// * `project_name` - The name of the project to create.
    pub async fn create_project(&self, project_name: &str) {
        match self {
            VectorStore::SQLiteStore(conn) => {
                SQLite::create_table(conn, &project_name).unwrap();
            }
        }
    }

    /// Deletes a project from the vector store.
    ///
    /// # Arguments
    ///
    /// * `self` - The `VectorStore` to delete the project from.
    /// * `project_name` - The name of the project to delete.
    pub async fn delete_project(&self, project_name: &str) {
        match self {
            VectorStore::SQLiteStore(conn) => {
                SQLite::delete_project(conn, project_name).unwrap();
            }
        }
    }

    /// Checks if a project exists in the vector store.
    ///
    /// # Arguments
    ///
    /// * `self` - The `VectorStore` to check for the project in.
    /// * `project_name` - The name of the project to check for existence.
    ///
    /// # Returns
    ///
    /// `true` if a project with the given name exists in the vector store, `false` otherwise.
    pub async fn does_project_exist(&self, project_name: &str) -> bool {
        match self {
            VectorStore::SQLiteStore(conn) => {
                SQLite::does_project_exist(conn, project_name).unwrap()
            }
        }
    }

    /// Retrieves information about a project from the vector store.
    ///
    /// # Arguments
    ///
    /// * `self` - The `VectorStore` to retrieve project information from.
    /// * `project_name` - The name of the project to retrieve information for.
    ///
    /// # Returns
    ///
    /// A `ProjectInfo` struct containing information about the project, or `None` if the project doesn't exist.
    pub async fn get_project_info(&self, project_name: &str) -> Option<ProjectInfo> {
        match self {
            VectorStore::SQLiteStore(conn) => SQLite::get_project_info(conn, project_name).unwrap(),
        }
    }

    /// Inserts code blocks and their embeddings into a project in the vector store.
    ///
    /// # Arguments
    ///
    /// * `self` - The `VectorStore` to insert blocks into.
    /// * `project_name` - The name of the project to insert the blocks into.
    /// * `blocks` - The list of `EmbeddedBlock` structs to insert, containing code blocks and their vector embeddings.
    pub async fn insert_blocks(&mut self, project_name: &str, blocks: Vec<EmbeddedBlock>) {
        match self {
            VectorStore::SQLiteStore(conn) => {
                SQLite::insert_blocks(conn, project_name, blocks).unwrap();
            }
        }
    }

    /// Searches for code blocks in a project that match a query code, using vector embeddings.
    ///
    /// # Arguments
    ///
    /// * `self` - The `VectorStore` to search.
    /// * `project_name` - The name of the project to search in.
    /// * `search_code` - The code to search for matching blocks to.
    ///
    /// # Returns
    ///
    /// A `NearestVectors` struct containing the most similar code block and a list of the nearest matching blocks.
    pub async fn search(&self, project_name: &str, search_code: String) -> NearestVectors {
        match self {
            VectorStore::SQLiteStore(conn) => {
                let code_vectors = SQLite::get_code_vectors(conn, project_name).unwrap();

                Embeddings::search(code_vectors, search_code, 5).unwrap()
            }
        }
    }

    /// Retrieves all code blocks from a project that are non-empty functions.
    ///  
    /// # Arguments
    ///
    /// * `self` - The `VectorStore` to retrieve blocks from.
    /// * `project_name` - The name of the project to retrieve blocks for.
    ///
    /// # Returns  
    ///
    /// A list of `Block` structs representing the code blocks that are non-empty functions.
    pub async fn get_all_function_blocks(&self, project_name: &str) -> Vec<asterisk::block::Block> {
        match self {
            VectorStore::SQLiteStore(conn) => {
                SQLite::get_all_function_blocks(conn, project_name).unwrap()
            }
        }
    }

    /// Searches for code blocks matching a query in a project, filtering for non-empty functions.
    ///
    /// # Arguments
    ///
    /// * `self` - The `VectorStore` to search.
    /// * `project_name` - The name of the project to search in.
    /// * `search_code` - The code to search for matches to.
    ///
    /// # Returns
    ///
    /// A list of `Block` structs representing the code blocks that match the query and are non-empty functions.
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

    /// Searches for code blocks with a specific function name in a project.
    ///
    /// # Arguments
    ///
    /// * `self` - The `VectorStore` to search.
    /// * `project_name` - The name of the project to search in.
    /// * `function_name` - The name of the function to search for.
    ///
    /// # Returns
    ///
    /// A list of `Block` structs representing the code blocks with the given function name.
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
