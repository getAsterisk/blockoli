<img src=".assets/blockoli.png" alt="blockoli logo" width="250" align="right">

# `blockoli` 🥦🔎

Blockoli is a high-performance tool for code indexing, embedding generation and semantic search tool for use with LLMs. blockoli is built in Rust and uses the [ASTerisk](https://github.com/stitionai/asterisk) crate for semantic code parsing. blockoli allows you to efficiently index, store, and search code blocks and their embeddings using vector similarity.

## Features
- Index code blocks from a codebase 📂🔍
- Generate vector embeddings for code blocks using a pre-trained model 🤖🧠
- Store code blocks and their embeddings in a SQLite database (Support for Qdrant soon!) 💾🗄️
- Perform efficient similarity search on code blocks using vector embeddings (k-d tree algorithm) 🔎⚡
- REST API for easy integration with other tools and platforms 🌐🔗
- Fast and memory-efficient implementation using Rust ⚡💻

## Installation (from source)

1. Ensure you have Rust installed on your system. You can install it from the official Rust website: https://www.rust-lang.org/tools/install

2. Clone the blockoli repository:

```bash
git clone https://github.com/stitionai/blockoli.git
cd blockoli
```

3. Download `tree-sitter` grammar files
```
mkdir grammars
chmod +x get-grammar.sh
./get-grammar.sh
```

4. Build the project:

```bash
cargo build --release
```

5. Run the server:

```bash
./target/release/blockoli <port>
```

Replace `<port>` with the desired port number for the server.

## Usage

Blockoli provides a REST API for indexing and searching code blocks. Here are some example API endpoints:

- `POST /project`: Create a new project
- `GET /project/{project_name}`: Get information about a project
- `DELETE /project/{project_name}`: Delete a project
- `POST /project/generate`: Generate embeddings for code blocks in a project
- `POST /search/{code_block}`: Search for similar code blocks in a project
- `POST /get_blocks/{project_name}`: Get all function blocks in a project
- `POST /search_blocks/{function_block}`: Search for function blocks in a project
- `POST /search_by_function/{function_name}`: Search for blocks by function name in a project

Refer to the `routes.rs` file for detailed information about each API endpoint and its parameters.

## Configuration

`ASTerisk` uses a configuration file named `asterisk.toml` for specifying indexing options. Modify this file to customize the behavior of the indexer according to your needs.

## Contribution Guidelines

Contributions to Blockoli are welcome! If you find a bug, have a feature request, or want to contribute code improvements, please open an issue or submit a pull request on the GitHub repository.

When contributing code, please ensure that your changes are well-tested and follow the Rust coding conventions and style guidelines.

## Contribution

Ways to contribute:
- Suggest a feature
- Report a bug
- Fix something and open a pull request
- Help document the code
- Spread the word

## License

Licensed under the MIT License, see <a href="https://github.com/stitionai/blockoli/blob/master/LICENSE">LICENSE</a> for more information.

## Liked the project?

Support the project by starring the repository. ⭐

---
