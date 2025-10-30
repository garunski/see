use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FileKind {
    Rust,
    JsTs,
    Css,
    Toml,
    Html,
    Other,
}

fn kind_for_path(path: &Path) -> FileKind {
    match path.extension().and_then(|s| s.to_str()).unwrap_or("") {
        "rs" => FileKind::Rust,
        "ts" | "tsx" | "js" | "jsx" => FileKind::JsTs,
        "css" => FileKind::Css,
        "toml" => FileKind::Toml,
        "html" | "htm" => FileKind::Html,
        _ => FileKind::Other,
    }
}

fn should_skip_dir(dir: &Path) -> bool {
    let name = dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
    matches!(
        name,
        "target" | "node_modules" | "dist" | "build" | "coverage" |
        ".git" | ".cargo" | "dx"
    )
}

fn strip_comments_from_content(content: &str, kind: FileKind) -> String {
    match kind {
        FileKind::Rust | FileKind::JsTs | FileKind::Css => strip_c_like_comments(content, kind),
        FileKind::Toml => strip_toml_comments(content),
        FileKind::Html => strip_html_comments(content),
        FileKind::Other => content.to_string(),
    }
}

fn strip_html_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 3 < bytes.len() && &bytes[i..i + 4] == b"<!--" {

            i += 4;
            while i + 2 < bytes.len() {
                if &bytes[i..i + 3] == b"-->" {
                    i += 3;
                    break;
                }
                i += 1;
            }
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn strip_toml_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for line in input.lines() {
        let mut in_basic = false;
        let mut in_literal = false;
        let mut escaped = false;
        let mut trimmed = String::with_capacity(line.len());
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            if in_basic {
                if !escaped && c == '\\' {
                    escaped = true;
                    trimmed.push(c);
                    continue;
                }
                if !escaped && c == '"' {
                    in_basic = false;
                }
                escaped = false;
                trimmed.push(c);
                continue;
            }
            if in_literal {
                if c == '\'' {
                    in_literal = false;
                }
                trimmed.push(c);
                continue;
            }
            if c == '"' {
                in_basic = true;
                trimmed.push(c);
                continue;
            }
            if c == '\'' {
                in_literal = true;
                trimmed.push(c);
                continue;
            }
            if c == '#' {

                break;
            }
            trimmed.push(c);
        }
        out.push_str(trimmed.trim_end());
        out.push('\n');
    }
    out
}

fn strip_c_like_comments(input: &str, kind: FileKind) -> String {

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum StrKind { None, Single, Double, Backtick, RustRaw(usize) }

    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    let mut str_kind = StrKind::None;
    let mut escaped = false;
    let mut block_depth = 0usize;

    while i < bytes.len() {
        let b = bytes[i];


        if block_depth > 0 {
            if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
                block_depth += 1;
                i += 2;
                continue;
            }
            if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b'/' {
                block_depth -= 1;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }

        match str_kind {
            StrKind::None => {

                if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {

                    i += 2;
                    while i < bytes.len() {
                        if bytes[i] == b'\n' { out.push(b'\n'); i += 1; break; }
                        i += 1;
                    }
                    continue;
                }
                if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
                    block_depth = 1;
                    i += 2;
                    continue;
                }

                if matches!(kind, FileKind::Rust) {
                    if i + 2 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' && bytes[i + 2] == b'/' {

                        i += 3;
                        while i < bytes.len() {
                            if bytes[i] == b'\n' { out.push(b'\n'); i += 1; break; }
                            i += 1;
                        }
                        continue;
                    }
                    if i + 2 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'!' && bytes[i + 2] == b'!' {
                        i += 3;
                        while i < bytes.len() {
                            if bytes[i] == b'\n' { out.push(b'\n'); i += 1; break; }
                            i += 1;
                        }
                        continue;
                    }
                }


                if b == b'\'' { str_kind = StrKind::Single; out.push(b); i += 1; continue; }
                if b == b'"' { str_kind = StrKind::Double; out.push(b); i += 1; continue; }
                if matches!(kind, FileKind::JsTs) && b == b'`' { str_kind = StrKind::Backtick; out.push(b); i += 1; continue; }


                if matches!(kind, FileKind::Rust) && b == b'r' {
                    let mut j = i + 1;
                    let mut hashes = 0usize;
                    while j < bytes.len() && bytes[j] == b'#' { hashes += 1; j += 1; }
                    if j < bytes.len() && bytes[j] == b'"' {

                        str_kind = StrKind::RustRaw(hashes);

                        while i <= j { out.push(bytes[i]); i += 1; }
                        continue;
                    }
                }


                out.push(b);
                i += 1;
            }
            StrKind::Single => {
                out.push(b);
                if !escaped && b == b'\\' { escaped = true; i += 1; continue; }
                if !escaped && b == b'\'' { str_kind = StrKind::None; }
                escaped = false;
                i += 1;
            }
            StrKind::Double => {
                out.push(b);
                if !escaped && b == b'\\' { escaped = true; i += 1; continue; }
                if !escaped && b == b'"' { str_kind = StrKind::None; }
                escaped = false;
                i += 1;
            }
            StrKind::Backtick => {
                out.push(b);
                if b == b'`' { str_kind = StrKind::None; }
                i += 1;
            }
            StrKind::RustRaw(hashes) => {
                out.push(b);
                if b == b'"' {

                    let mut k = i + 1;
                    let mut count = 0usize;
                    while k < bytes.len() && bytes[k] == b'#' && count < hashes { count += 1; k += 1; }
                    if count == hashes {
                        str_kind = StrKind::None;
                    }
                }
                i += 1;
            }
        }
    }

    String::from_utf8(out).unwrap_or_default()
}

fn iter_files(root: &Path, acc: &mut Vec<PathBuf>) -> io::Result<()> {
    if should_skip_dir(root) { return Ok(()); }
    if root.is_dir() {
        for entry in fs::read_dir(root)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if should_skip_dir(&path) { continue; }
                iter_files(&path, acc)?;
            } else {
                acc.push(path);
            }
        }
    }
    Ok(())
}

fn process_file(path: &Path) -> io::Result<bool> {
    let kind = kind_for_path(path);
    if kind == FileKind::Other { return Ok(false); }
    let mut file = fs::File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let stripped = strip_comments_from_content(&content, kind);
    if stripped != content {
        let mut f = fs::File::create(path)?;
        f.write_all(stripped.as_bytes())?;
        return Ok(true);
    }
    Ok(false)
}

fn main() -> io::Result<()> {
    let cwd = std::env::current_dir()?;
    let mut files = Vec::new();
    iter_files(&cwd, &mut files)?;

    let mut modified = 0usize;
    for path in files {

        if path.components().any(|c| {
            if let std::path::Component::Normal(s) = c {
                matches!(s.to_str().unwrap_or(""), "target" | "node_modules" | "dist" | "build" | "coverage" | ".git" | ".cargo" | "dx")
            } else { false }
        }) { continue; }

        match process_file(&path) {
            Ok(true) => { modified += 1; }
            Ok(false) => {}
            Err(err) => {
                eprintln!("Failed to process {}: {}", path.display(), err);
            }
        }
    }

    eprintln!("Stripped comments from {} files", modified);
    Ok(())
}


