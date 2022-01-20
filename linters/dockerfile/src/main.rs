mod lints;

use aspen::{tree_sitter::Language, Linter};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref DOCKERFILE: Language = tree_sitter_dockerfile::language();
}

fn main() {
    let linter = Linter::new(*DOCKERFILE)
        .lints(lints::lints())
        .comment_str("#");
    linter.run_analysis();
}