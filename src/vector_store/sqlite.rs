use anyhow::Result;
use asterisk::block::{Block, BlockType};
use rusqlite::{params, Connection};
use serde::Serialize;

use indicatif::{ProgressBar, ProgressStyle};

use crate::{blocks::EmbeddedBlock, embeddings::encoder::Vector};

#[derive(Clone)]
pub struct SQLite {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub code: String,
    /// If the code is a function, this will be the function name
    pub function_name: Option<String>,
    pub vectors: Vec<f32>,
    pub incoming_calls: Vec<String>,
    pub outgoing_calls: Vec<String>,
}

#[derive(Serialize)]
pub struct ProjectInfo {
    pub name: String,
    pub total_code_blocks: i32,
}

impl SQLite {
    fn validate_project_name(project_name: &str) {
        if !project_name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_')
        {
            panic!("Project name must be alphanumeric or underscore characters only");
        }
    }

    pub fn does_project_exist(conn: &Connection, project_name: &str) -> Result<bool> {
        Self::validate_project_name(project_name);
        let query = "SELECT name FROM sqlite_master WHERE type='table' AND name=?";
        let mut stmt = conn.prepare(query)?;
        let mut rows = stmt.query(params![project_name])?;

        if let Some(_) = rows.next()? {
            return Ok(true);
        } else {
            return Ok(false);
        }
    }

    pub fn create_table(conn: &Connection, project_name: &str) -> Result<()> {
        Self::validate_project_name(project_name);
        let query = format!(
            "CREATE TABLE IF NOT EXISTS {} (
            id INTEGER PRIMARY KEY,
            node_key TEXT NOT NULL,
            block_type TEXT NOT NULL,
            content TEXT NOT NULL,
            class_name TEXT NOT NULL,
            function_name TEXT NOT NULL,
            outgoing_calls TEXT NOT NULL,
            vectors TEXT NOT NULL
        )",
            project_name
        );

        conn.execute(&query, params![])?;

        Ok(())
    }

    pub fn delete_project(conn: &Connection, project_name: &str) -> Result<()> {
        Self::validate_project_name(project_name);
        let query = format!("DROP TABLE IF EXISTS {}", project_name);
        conn.execute(&query, params![])?;
        conn.execute("VACUUM", params![])?;
        Ok(())
    }

    pub fn get_project_info(conn: &Connection, project_name: &str) -> Result<Option<ProjectInfo>> {
        Self::validate_project_name(project_name);
        let query = format!("SELECT COUNT(*) FROM {}", project_name);

        if let Ok(total_code_blocks) = conn.query_row(&query, params![], |row| row.get(0)) {
            return Ok(Some(ProjectInfo {
                name: project_name.to_owned(),
                total_code_blocks,
            }));
        } else {
            return Ok(None);
        }
    }

    pub fn insert_blocks(
        conn: &mut Connection,
        project_name: &str,
        blocks: Vec<EmbeddedBlock>,
    ) -> Result<()> {
        Self::validate_project_name(project_name);

        eprintln!("\n[-] Inserting blocks into {}", project_name);

        let progress_bar = ProgressBar::new(blocks.len() as u64);
        progress_bar.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {percent}% {per_sec} ETA: {eta}"));

        let transaction = conn.transaction()?;
        let query = format!(
            "INSERT INTO {} (node_key, block_type, content, class_name, function_name, outgoing_calls, vectors) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            project_name
        );

        for block in blocks {
            transaction.execute(
                &query,
                params![
                    block.block.node_key,
                    serde_json::to_string(&block.block.block_type).unwrap(),
                    block.block.content,
                    serde_json::to_string(&block.block.class_name.clone()).unwrap(),
                    serde_json::to_string(&block.block.function_name.clone()).unwrap(),
                    serde_json::to_string(&block.block.outgoing_calls).unwrap(),
                    serde_json::to_string(&block.vectors).unwrap(),
                ],
            )?;

            progress_bar.inc(1);
        }

        transaction.commit()?;
        progress_bar.finish();

        Ok(())
    }

    pub fn get_all_function_blocks(
        conn: &Connection,
        project_name: &str,
    ) -> Result<Vec<asterisk::block::Block>> {
        Self::validate_project_name(project_name);
        let query = format!("SELECT * FROM {} WHERE function_name != ''", project_name);
        let mut stmt = conn.prepare(&query)?;
        let blocks_iter = stmt.query_map(params![], |row| {
            let block_type_string = row.get::<_, String>(2)?;
            let block_type = serde_json::from_str(&block_type_string).unwrap();

            let class_name_string = row.get::<_, String>(4)?;
            let class_name = serde_json::from_str(&class_name_string).unwrap_or_default();

            let function_name_string = row.get::<_, String>(5)?;
            let function_name = serde_json::from_str(&function_name_string).unwrap_or_default();

            let outgoing_calls_string = row.get::<_, String>(6)?;
            let outgoing_calls: Vec<String> = serde_json::from_str(&outgoing_calls_string).unwrap();

            Ok(asterisk::block::Block {
                node_key: row.get(1)?,
                block_type,
                content: row.get(3)?,
                class_name,
                function_name,
                outgoing_calls,
            })
        })?;

        let mut blocks = Vec::new();

        for project in blocks_iter {
            blocks.push(project?);
        }

        Ok(blocks)
    }

    pub fn search_from_function_blocks(
        conn: &Connection,
        project_name: &str,
        search_code: &str,
    ) -> Result<Vec<asterisk::block::Block>> {
        Self::validate_project_name(project_name);
        let query = format!(
            "SELECT * FROM {} WHERE function_name != '' AND code LIKE ?",
            project_name
        );
        let mut stmt = conn.prepare(&query)?;
        let blocks_iter = stmt.query_map(params!["%".to_owned() + search_code + "%"], |row| {
            let block_type_string = row.get::<_, String>(2)?;
            let block_type = serde_json::from_str(&block_type_string).unwrap();

            let class_name_string = row.get::<_, String>(4)?;
            let class_name = serde_json::from_str(&class_name_string).unwrap_or_default();

            let function_name_string = row.get::<_, String>(5)?;
            let function_name = serde_json::from_str(&function_name_string).unwrap_or_default();

            let outgoing_calls_string = row.get::<_, String>(6)?;
            let outgoing_calls: Vec<String> = serde_json::from_str(&outgoing_calls_string).unwrap();

            Ok(asterisk::block::Block {
                node_key: row.get(1)?,
                block_type,
                content: row.get(3)?,
                class_name,
                function_name,
                outgoing_calls,
            })
        })?;

        let mut blocks = Vec::new();

        for project in blocks_iter {
            blocks.push(project?);
        }

        Ok(blocks)
    }

    pub fn search_by_function_name(
        conn: &Connection,
        project_name: &str,
        function_name: &str,
    ) -> Result<Vec<asterisk::block::Block>> {
        Self::validate_project_name(project_name);
        let query = format!(
            "SELECT * FROM {} WHERE function_name != '' AND function_name = ?",
            project_name,
        );
        let mut stmt = conn.prepare(&query)?;
        let blocks_iter = stmt.query_map(params![function_name], |row| {
            let block_type_string = row.get::<_, String>(2)?;
            let block_type = serde_json::from_str(&block_type_string).unwrap();

            let class_name_string = row.get::<_, String>(4)?;
            let class_name = serde_json::from_str(&class_name_string).unwrap_or_default();

            let function_name_string = row.get::<_, String>(5)?;
            let function_name = serde_json::from_str(&function_name_string).unwrap_or_default();

            let outgoing_calls_string = row.get::<_, String>(6)?;
            let outgoing_calls: Vec<String> = serde_json::from_str(&outgoing_calls_string).unwrap();

            Ok(asterisk::block::Block {
                node_key: row.get(1)?,
                block_type,
                content: row.get(3)?,
                class_name,
                function_name,
                outgoing_calls,
            })
        })?;

        let mut blocks = Vec::new();

        for project in blocks_iter {
            blocks.push(project?);
        }

        Ok(blocks)
    }

    pub fn get_code_vectors(conn: &Connection, project_name: &str) -> Result<Vec<Vector>> {
        Self::validate_project_name(project_name);
        let query = format!("SELECT * FROM {}", project_name);
        let mut stmt = conn.prepare(&query)?;
        let project_iter = stmt.query_map(params![], |row| {
            let content: String = row.get(2)?;

            let vectors_string = row.get::<_, String>(7)?;
            let vectors: Vec<f32> = serde_json::from_str(&vectors_string).unwrap();

            Ok((vectors, content))
        })?;

        let mut code_vectors = Vec::new();

        for project in project_iter {
            let project = project?;
            let code_vector = Vector {
                point: project.0.as_slice().try_into().unwrap(),
                code: project.1,
            };

            code_vectors.push(code_vector);
        }

        Ok(code_vectors)
    }
}
