#[derive(Debug, Clone, PartialEq)]
pub enum DiffLineKind {
    FileHeader,    // --- a/file, +++ b/file lines
    HunkHeader,    // @@ -10,5 +10,7 @@ fn foo()
    Addition,      // Lines starting with +
    Deletion,      // Lines starting with -
    Context,       // Lines starting with space (unchanged)
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub old_lineno: Option<usize>,
    pub new_lineno: Option<usize>,
    pub content: String,
}

pub fn parse_unified_diff(raw: &str) -> Vec<DiffLine> {
    let mut result = Vec::new();
    let mut old_lineno = 0;
    let mut new_lineno = 0;

    for line in raw.lines() {
        if line.starts_with("---") || line.starts_with("+++") {
            result.push(DiffLine {
                kind: DiffLineKind::FileHeader,
                old_lineno: None,
                new_lineno: None,
                content: line.to_string(),
            });
        } else if line.starts_with("@@") {
            if let Some((old_st, new_st)) = parse_hunk_header(line) {
                old_lineno = old_st;
                new_lineno = new_st;
            }
            result.push(DiffLine {
                kind: DiffLineKind::HunkHeader,
                old_lineno: None,
                new_lineno: None,
                content: line.to_string(),
            });
        } else if line.starts_with('+') {
            result.push(DiffLine {
                kind: DiffLineKind::Addition,
                old_lineno: None,
                new_lineno: Some(new_lineno),
                content: line[1..].to_string(),
            });
            new_lineno += 1;
        } else if line.starts_with('-') {
            result.push(DiffLine {
                kind: DiffLineKind::Deletion,
                old_lineno: Some(old_lineno),
                new_lineno: None,
                content: line[1..].to_string(),
            });
            old_lineno += 1;
        } else if line.starts_with(' ') {
            result.push(DiffLine {
                kind: DiffLineKind::Context,
                old_lineno: Some(old_lineno),
                new_lineno: Some(new_lineno),
                content: line[1..].to_string(),
            });
            old_lineno += 1;
            new_lineno += 1;
        } else if line.starts_with("\\ ") {
            result.push(DiffLine {
                kind: DiffLineKind::Context,
                old_lineno: None,
                new_lineno: None,
                content: line.to_string(),
            });
        } else {
            result.push(DiffLine {
                kind: DiffLineKind::Context,
                old_lineno: None,
                new_lineno: None,
                content: line.to_string(),
            });
        }
    }

    result
}

fn parse_hunk_header(line: &str) -> Option<(usize, usize)> {
    if !line.starts_with("@@ ") {
        return None;
    }

    let end_idx = line[3..].find(" @@")?;
    let inner = &line[3..3 + end_idx];

    let mut parts = inner.split_whitespace();
    let old_part = parts.next()?;
    let new_part = parts.next()?;

    let parse_part = |s: &str, prefix: char| -> Option<usize> {
        let s = s.strip_prefix(prefix)?;
        let num_str = match s.find(',') {
            Some(idx) => &s[..idx],
            None => s,
        };
        num_str.parse().ok()
    };

    let old_st = parse_part(old_part, '-')?;
    let new_st = parse_part(new_part, '+')?;

    Some((old_st, new_st))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_diff() {
        let diff = r#"--- a/src/main.rs
+++ b/src/main.rs
@@ -10,3 +10,4 @@
 fn foo() {
-    println!("old");
+    println!("new");
+    println!("more");
 }"#;

        let lines = parse_unified_diff(diff);
        assert_eq!(lines.len(), 8);

        assert_eq!(lines[0].kind, DiffLineKind::FileHeader);
        assert_eq!(lines[1].kind, DiffLineKind::FileHeader);
        assert_eq!(lines[2].kind, DiffLineKind::HunkHeader);

        // fn foo() {
        assert_eq!(lines[3].kind, DiffLineKind::Context);
        assert_eq!(lines[3].old_lineno, Some(10));
        assert_eq!(lines[3].new_lineno, Some(10));
        assert_eq!(lines[3].content, "fn foo() {");

        // -    println!("old");
        assert_eq!(lines[4].kind, DiffLineKind::Deletion);
        assert_eq!(lines[4].old_lineno, Some(11));
        assert_eq!(lines[4].new_lineno, None);
        assert_eq!(lines[4].content, r#"    println!("old");"#);

        // +    println!("new");
        assert_eq!(lines[5].kind, DiffLineKind::Addition);
        assert_eq!(lines[5].old_lineno, None);
        assert_eq!(lines[5].new_lineno, Some(11));
        assert_eq!(lines[5].content, r#"    println!("new");"#);

        // +    println!("more");
        assert_eq!(lines[6].kind, DiffLineKind::Addition);
        assert_eq!(lines[6].old_lineno, None);
        assert_eq!(lines[6].new_lineno, Some(12));
        assert_eq!(lines[6].content, r#"    println!("more");"#);

        // }
        assert_eq!(lines[7].kind, DiffLineKind::Context);
        assert_eq!(lines[7].old_lineno, Some(12));
        assert_eq!(lines[7].new_lineno, Some(13));
        assert_eq!(lines[7].content, "}");
    }

    #[test]
    fn test_parse_multiple_hunks() {
        let diff = "\
@@ -1,2 +1,2 @@
 a
-b
+c
@@ -10,2 +10,2 @@
 x
-y
+z";
        
        let lines = parse_unified_diff(diff);
        assert_eq!(lines.len(), 8);
        
        // Second hunk header
        assert_eq!(lines[4].kind, DiffLineKind::HunkHeader);
        
        // x
        assert_eq!(lines[5].kind, DiffLineKind::Context);
        assert_eq!(lines[5].old_lineno, Some(10));
        assert_eq!(lines[5].new_lineno, Some(10));
        
        // -y
        assert_eq!(lines[6].kind, DiffLineKind::Deletion);
        assert_eq!(lines[6].old_lineno, Some(11));
        assert_eq!(lines[6].new_lineno, None);
        
        // +z
        assert_eq!(lines[7].kind, DiffLineKind::Addition);
        assert_eq!(lines[7].old_lineno, None);
        assert_eq!(lines[7].new_lineno, Some(11));
    }

    #[test]
    fn test_parse_hunk_header_extraction() {
        assert_eq!(parse_hunk_header("@@ -10,5 +10,7 @@ fn foo()"), Some((10, 10)));
        assert_eq!(parse_hunk_header("@@ -1 +2 @@"), Some((1, 2)));
        assert_eq!(parse_hunk_header("@@ -42,1 +43,99 @@ test"), Some((42, 43)));
        assert_eq!(parse_hunk_header("invalid"), None);
        assert_eq!(parse_hunk_header("@@ invalid @@"), None);
    }

    #[test]
    fn test_empty_diff() {
        let lines = parse_unified_diff("");
        assert!(lines.is_empty());
    }
}
