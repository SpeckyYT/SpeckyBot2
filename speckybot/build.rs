use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use proc_macro2::{TokenStream, TokenTree};
use syn::{Expr, File, Item, Lit};

#[derive(Clone, Debug)]
struct NameOccurrence {
    name: String,
    file: String,
    line: usize,
    column: usize,
    source_line: String,
}

#[macro_export]
macro_rules! output_file {
    () => { "commands_modules.rs" };
}

pub fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(out_dir).join(output_file!());
    let rs_files = collect_rs_files("src/commands/").unwrap_or_default();
    check_for_duplicate_command_names(&rs_files);
    let module_code = generate_module_declarations(&rs_files);

    let mut file = fs::File::create(&out_path).unwrap();
    file.write_all(module_code.as_bytes()).unwrap();

    println!("cargo:rerun-if-changed=src/commands/");
}

pub fn collect_rs_files(base_dir: &str) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    collect_rs_files_recursive(base_dir, base_dir, &mut files)?;
    files.sort();
    Ok(files)
}

pub fn collect_rs_files_recursive(
    base_dir: &str,
    current_dir: &str,
    files: &mut Vec<(String, String)>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            if let Some(dir_name) = path.file_name() {
                if dir_name != "." && dir_name != ".." {
                    collect_rs_files_recursive(base_dir, path.to_str().unwrap(), files)?;
                }
            }
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            // Get relative path from base_dir
            if let Ok(string) = fs::read_to_string(&path) && string.starts_with("#[ignore]") {
                continue
            }

            if let Ok(relative_path) = path.strip_prefix(base_dir) {
                let rel_path_str = relative_path.to_string_lossy().to_string();
                
                if let Some(file_name) = path.file_name() {
                    if let Some(name) = file_name.to_str() {
                        // Skip mod.rs files
                        if name != "mod.rs" {
                            let path_without_ext = rel_path_str.trim_end_matches(".rs").to_string();
                            files.push((path_without_ext, rel_path_str));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn sanitize_ident(s: &str) -> Cow<'_, str> {
    let out = s.char_indices().fold(None, |acc, (i, c)| {
        let ok = c.is_ascii_alphanumeric() || c == '_';
        match acc {
            None if ok => None,
            None => {
                let mut buf = String::with_capacity(s.len());
                buf.push_str(&s[..i]);
                buf.push('_');
                Some(buf)
            }
            Some(mut buf) if ok => { buf.push(c); Some(buf) }
            Some(mut buf) => { buf.push('_'); Some(buf) }
        }
    });
    match out {
        None => Cow::Borrowed(s),
        Some(buf) => Cow::Owned(buf),
    }
}

fn parse_name_expr(expr: &Expr) -> Vec<String> {
    match expr {
        Expr::Array(array) => array
            .elems
            .iter()
            .flat_map(parse_name_expr)
            .collect(),
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Str(lit) => vec![lit.value()],
            _ => Vec::new(),
        },
        Expr::Group(group) => parse_name_expr(&group.expr),
        _ => Vec::new(),
    }
}

fn extract_command_names_from_tokens(tokens: TokenStream) -> Vec<String> {
    let mut values = Vec::new();
    let mut stream = tokens.into_iter().peekable();

    while let Some(token) = stream.next() {
        if let TokenTree::Ident(ident) = &token {
            if ident == "names" {
                if matches!(stream.peek(), Some(TokenTree::Punct(p)) if p.as_char() == ':') {
                    stream.next();
                }

                let mut expr_tokens = Vec::new();
                while let Some(next) = stream.next() {
                    if matches!(&next, TokenTree::Punct(p) if p.as_char() == ',') {
                        break;
                    }
                    expr_tokens.push(next);
                }

                let expr_stream = TokenStream::from_iter(expr_tokens);
                if let Ok(expr) = syn::parse2::<Expr>(expr_stream) {
                    values.extend(parse_name_expr(&expr));
                }
            }
        }
    }

    values
}

fn find_name_location(contents: &str, name: &str) -> Option<(usize, usize, String)> {
    let needle = format!("\"{name}\"");
    let offset = contents.find(&needle)?;
    let before = &contents[..offset];
    let line = before.lines().count();
    let line_start = before.rfind('\n').map_or(0, |idx| idx + 1);
    let column = offset - line_start + 1;
    let source_line = contents
        .lines()
        .nth(line.saturating_sub(1))
        .unwrap_or("")
        .trim_end()
        .to_string();
    Some((line, column, source_line))
}

fn extract_command_names(contents: &str, file_path: &str) -> Vec<NameOccurrence> {
    let parsed: File = match syn::parse_str(contents) {
        Ok(file) => file,
        Err(_) => return Vec::new(),
    };

    let mut names = Vec::new();
    for item in parsed.items {
        if let Item::Macro(item_macro) = item {
            let path = item_macro.mac.path.segments.last().map(|segment| segment.ident.to_string());
            if path.as_deref() == Some("command") {
                for name in extract_command_names_from_tokens(item_macro.mac.tokens) {
                    let (line, column, source_line) = find_name_location(contents, &name).unwrap_or((1, 1, String::new()));
                    names.push(NameOccurrence {
                        name,
                        file: file_path.to_string(),
                        line,
                        column,
                        source_line,
                    });
                }
            }
        }
    }

    names
}

fn check_for_duplicate_command_names(files: &[(String, String)]) {
    let mut seen: HashMap<String, Vec<NameOccurrence>> = HashMap::new();

    for (relative_path, _) in files {
        let full_path = PathBuf::from("src/commands/").join(format!("{relative_path}.rs"));
        let source = match fs::read_to_string(&full_path) {
            Ok(source) => source,
            Err(_) => continue,
        };

        for name in extract_command_names(&source, &relative_path.replace('\\', "/")) {
            seen.entry(name.name.clone()).or_default().push(name);
        }
    }

    let duplicates: Vec<(String, Vec<NameOccurrence>)> = seen
        .into_iter()
        .filter(|(_, occurrences)| occurrences.len() > 1)
        .collect();

    if !duplicates.is_empty() {
        let mut error_msg = String::from("\n=== Duplicate Command Names Detected ===\n\n");
        for (name, occurrences) in duplicates {
            error_msg.push_str(&format!("error: duplicate command name `{name}`\n"));
            for occurrence in occurrences {
                let NameOccurrence { file, line, column, source_line, .. } = occurrence;
                let empty = String::new();
                error_msg.push_str(&format!(
                    "  --> src/commands/{file}.rs:{line}:{column}\n {empty:>4} |\n {line:>4} | {source_line}\n {empty:>4} | {}^\n\n",
                    " ".repeat(occurrence.column - 1),
                ));
            }
        }
        panic!("{}", error_msg);
    }
}

pub fn generate_module_declarations(files: &[(String, String)]) -> String {
    let mut declarations = "commands![".to_string();

    for (rel_path, file_path) in files {
        let module_name = sanitize_ident(rel_path);

        declarations.push_str(&format!(
            r#"{module_name}:"{}/src/commands/{}","#,
            env!("CARGO_MANIFEST_DIR").replace("\\", "/"),
            file_path.replace("\\", "/"),
        ));
        declarations.push('\n');
    }
    declarations.push_str("];");
    
    declarations
}
