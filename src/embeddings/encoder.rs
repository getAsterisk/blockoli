use fastembed::{EmbeddingBase, FlagEmbedding};

use anyhow::Result;
use once_cell::sync::Lazy;
use serde::Serialize;

pub static MODEL: Lazy<FlagEmbedding> =
    Lazy::new(|| FlagEmbedding::try_new(Default::default()).unwrap());

use kd_tree::{KdPoint, KdTree, KdTreeN};

pub const VECTOR_SIZE: usize = 384;

#[derive(Debug, Clone)]
pub struct Vector {
    pub point: [f32; VECTOR_SIZE],
    pub code: String,
}

impl KdPoint for Vector {
    type Scalar = f32;
    type Dim = typenum::U384;
    fn at(&self, k: usize) -> f32 {
        self.point[k]
    }
}

pub type VectorKdTree = KdTreeN<Vector, typenum::U384>;

#[derive(Debug, Clone)]
pub struct Embeddings {
    pub vector_set: Vec<Vector>,
    pub kd_tree: VectorKdTree,
}

#[derive(Serialize, Debug)]
pub struct NearestVectors {
    pub nearest: String,
    pub k_nearest: Vec<String>,
}

impl Embeddings {
    /// Generates a vector embedding for a given code block.
    ///
    /// # Arguments
    ///
    /// * `code` - The code block to generate an embedding for.
    ///
    /// # Returns
    ///
    /// A `Vector` struct containing the generated embedding and original code block.
    ///
    /// # Errors
    ///
    /// Returns an error if the embedding model fails to generate a vector.
    pub fn generate_code_vector(code: String) -> Result<Vector> {
        let mut code = code;

        let output = MODEL.embed(vec![code.to_owned()], None)?;
        let vector: [f32; VECTOR_SIZE] = output[0].as_slice().try_into().unwrap();

        Ok(Vector {
            point: vector,
            code: code,
        })
    }

    /// Generates a set of vector embeddings for a list of code blocks.
    ///
    /// # Arguments
    ///
    /// * `code_blocks` - A list of code blocks to generate embeddings for.
    ///
    /// # Returns
    ///
    /// A `Vec<Vector>` containing the generated embeddings paired with their original code blocks.
    ///
    /// # Errors
    ///
    /// Returns an error if the embedding model fails to generate any of the vectors.
    pub fn generate_vector_set(code_blocks: Vec<String>) -> Result<Vec<Vector>> {
        let output: Vec<Vec<f32>> = MODEL.embed(code_blocks.to_owned(), None)?;

        let vector_set: Vec<Vector> = output
            .iter()
            .zip(code_blocks.iter())
            .map(|(x, y)| Vector {
                point: x.as_slice().try_into().unwrap(),
                code: y.clone(),
            })
            .collect();

        Ok(vector_set)
    }

    /// Generates an `Embeddings` struct containing vector embeddings and a KD tree index for a list of code blocks.
    ///
    /// # Arguments
    ///
    /// * `code_blocks` - A list of code blocks to generate embeddings for.
    ///
    /// # Returns
    ///
    /// An `Embeddings` struct containing the generated embeddings and KD tree index.
    ///
    /// # Errors
    ///
    /// Returns an error if the embedding model fails to generate any of the vectors.
    pub fn _generate_embeddings(code_blocks: Vec<String>) -> Result<Self> {
        let output: Vec<Vec<f32>> = MODEL.embed(code_blocks.to_owned(), None)?;

        let vector_set: Vec<Vector> = output
            .iter()
            .zip(code_blocks.iter())
            .map(|(x, y)| Vector {
                point: x.as_slice().try_into().unwrap(),
                code: y.clone(),
            })
            .collect();

        let kdtree: VectorKdTree = KdTree::par_build_by_ordered_float(vector_set.to_owned());

        Ok(Embeddings {
            vector_set: vector_set,
            kd_tree: kdtree,
        })
    }

    /// Searches an `Embeddings` struct for the closest matches to a given code block.
    ///
    /// # Arguments
    ///
    /// * `self` - The `Embeddings` struct to search.
    /// * `text` - The code block to search for matches to.
    /// * `matches` - The number of closest matches to return.
    ///
    /// # Returns
    ///
    /// A `NearestVectors` struct containing the closest matching code block and a list of the top `matches` closest matches.
    ///
    /// # Errors
    ///
    /// Returns an error if the embedding model fails to generate a vector for the search query.
    pub fn _search_embeddings(self, text: String, matches: usize) -> Result<NearestVectors> {
        let query: Vector = Self::generate_code_vector(text)?;

        let nearest = self.kd_tree.nearest(&query).unwrap();

        let mut code_blocks = Vec::new();
        let k_nearest = self.kd_tree.nearests(&query, matches);

        for nearest in k_nearest {
            code_blocks.push(nearest.item.code.to_owned());
        }

        Ok(NearestVectors {
            nearest: nearest.item.code.to_owned(),
            k_nearest: code_blocks,
        })
    }

    /// Searches a list of `Vector` structs for the closest matches to a given code block.
    ///
    /// # Arguments
    ///
    /// * `vector_set` - The list of `Vector` structs to search.
    /// * `code` - The code block to search for matches to.
    /// * `matches` - The number of closest matches to return.
    ///
    /// # Returns
    ///
    /// A `NearestVectors` struct containing the closest matching code block and a list of the top `matches` closest matches.
    pub fn search(vector_set: Vec<Vector>, code: String, matches: usize) -> Result<NearestVectors> {
        let query: Vector = Self::generate_code_vector(code)?;

        let kdtree: VectorKdTree = KdTree::par_build_by_ordered_float(vector_set.to_owned());

        let nearest = kdtree.nearest(&query).unwrap();

        let mut code_blocks = Vec::new();
        let k_nearest = kdtree.nearests(&query, matches);

        for nearest in k_nearest {
            code_blocks.push(nearest.item.code.to_owned());
        }

        Ok(NearestVectors {
            nearest: nearest.item.code.to_owned(),
            k_nearest: code_blocks,
        })
    }
}
