pub struct FileType {
    name: String,
    hl_opts: HighlightingOptions,
}

#[derive(Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct HighlightingOptions {
    numbers: bool,
    strings: bool,
    characters: bool,
    comments: bool,
    multiline_comments: bool,
    primary_keywords: Vec<String>,
    secondary_keywords: Vec<String>,
}

impl FileType {
    #[must_use]
    pub fn name(&self) -> String {
        self.name.clone()
    }
    #[must_use]
    pub fn highlighting_options(&self) -> &HighlightingOptions {
        &self.hl_opts
    }
    #[must_use]
    pub fn from(file_name: &str) -> Self {
        if std::path::Path::new(file_name)
            .extension()
            .map_or(false, |ext| ext.eq_ignore_ascii_case("rs")) {
            return Self {
                name: String::from("Rust"),
                hl_opts: HighlightingOptions {
                    numbers: true,
                    strings: true,
                    characters: true,
                    comments: true,
                    multiline_comments: true,
                    primary_keywords: vec![
                        "as".to_owned(),
                        "break".to_owned(),
                        "const".to_owned(),
                        "continue".to_owned(),
                        "crate".to_owned(),
                        "else".to_owned(),
                        "enum".to_owned(),
                        "extern".to_owned(),
                        "false".to_owned(),
                        "fn".to_owned(),
                        "for".to_owned(),
                        "if".to_owned(),
                        "impl".to_owned(),
                        "in".to_owned(),
                        "let".to_owned(),
                        "loop".to_owned(),
                        "match".to_owned(),
                        "mod".to_owned(),
                        "move".to_owned(),
                        "mut".to_owned(),
                        "pub".to_owned(),
                        "ref".to_owned(),
                        "return".to_owned(),
                        "self".to_owned(),
                        "Self".to_owned(),
                        "static".to_owned(),
                        "struct".to_owned(),
                        "super".to_owned(),
                        "trait".to_owned(),
                        "true".to_owned(),
                        "type".to_owned(),
                        "unsafe".to_owned(),
                        "use".to_owned(),
                        "where".to_owned(),
                        "while".to_owned(),
                        "dyn".to_owned(),
                        "abstract".to_owned(),
                        "become".to_owned(),
                        "box".to_owned(),
                        "do".to_owned(),
                        "final".to_owned(),
                        "macro".to_owned(),
                        "override".to_owned(),
                        "priv".to_owned(),
                        "typeof".to_owned(),
                        "unsized".to_owned(),
                        "virtual".to_owned(),
                        "yield".to_owned(),
                        "async".to_owned(),
                        "await".to_owned(),
                        "try".to_owned(),
                    ],
                    secondary_keywords: vec![
                        "bool".to_owned(),
                        "char".to_owned(),
                        "i8".to_owned(),
                        "i16".to_owned(),
                        "i32".to_owned(),
                        "i64".to_owned(),
                        "isize".to_owned(),
                        "u8".to_owned(),
                        "u16".to_owned(),
                        "u32".to_owned(),
                        "u64".to_owned(),
                        "usize".to_owned(),
                        "f32".to_owned(),
                        "f64".to_owned(),
                    ],
                },
            };
        }
        
        Self::default()
    }
}

impl Default for FileType {
    fn default() -> Self {
        Self {
            name: String::from("No filetype"),
            hl_opts: HighlightingOptions::default(),
        }
    }
}

impl HighlightingOptions {
    #[must_use]
    pub fn numbers(&self) -> bool {
        self.numbers
    }
    #[must_use]
    pub fn strings(&self) -> bool {
        self.strings
    }
    #[must_use]
    pub fn characters(&self) -> bool {
        self.characters
    }
    #[must_use]
    pub fn comments(&self) -> bool {
        self.comments
    }
    #[must_use]
    pub fn multiline_comments(&self) -> bool {
        self.multiline_comments
    }
    #[must_use]
    pub fn primary_keywords(&self) -> &Vec<String> {
        &self.primary_keywords
    }
    #[must_use]
    pub fn secondary_keywords(&self) -> &Vec<String> {
        &self.secondary_keywords
    }
}