use std::path::PathBuf;

fn main() {
    let dirs: Vec<PathBuf> = [
        &["grammars", "tree-sitter-typescript", "typescript", "src"][..],
        &["grammars", "tree-sitter-rust", "src"][..],
        &["grammars", "tree-sitter-python", "src"][..],
        &["grammars", "tree-sitter-javascript", "src"][..],
        &["grammars", "tree-sitter-c", "src"][..],
    ]
    .iter()
    .map(|path| path.iter().collect::<PathBuf>())
    .collect();

    let mut cc_build = cc::Build::new();

    for dir in dirs {
        cc_build
            .include(&dir)
            .file(dir.join("parser.c"));

        if !dir.ends_with("tree-sitter-c/src") {
            cc_build.file(dir.join("scanner.c"));
        }
    }

    cc_build.compile("tree-sitter-languages");
}
