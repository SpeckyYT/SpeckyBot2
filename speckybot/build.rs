use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;
use std::env;
use std::io::Write;

#[macro_export]
macro_rules! output_file {
    () => { "commands_modules.rs" };
}

pub fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(out_dir).join(output_file!());
    let rs_files = collect_rs_files("src/commands/").unwrap_or_default();
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
