use std::env;
use std::fs;
use std::path::PathBuf;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn main() {
    let path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("fixtures/0001-source-summary/input/Counter.tsx"));

    let source = fs::read_to_string(&path).expect("failed to read source file");

    let source_type = SourceType::from_path(&path)
        .unwrap_or_default()
        .with_typescript(true)
        .with_jsx(true);

    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, source_type).parse();

    println!("File: {}", path.display());
    println!("Errors: {}", ret.errors.len());

    for error in &ret.errors {
        println!("  {error:?}");
    }

    println!("Program parsed.");
}
