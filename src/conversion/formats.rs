use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum FormatCategory {
    Image,
    Document,
    Audio,
    Video,
    Data,
    Archive,
}

impl FormatCategory {
    pub fn label(&self) -> &'static str {
        match self {
            FormatCategory::Image => "Image",
            FormatCategory::Document => "Document",
            FormatCategory::Audio => "Audio",
            FormatCategory::Video => "Video",
            FormatCategory::Data => "Data",
            FormatCategory::Archive => "Archive",
        }
    }

    pub fn all() -> Vec<FormatCategory> {
        vec![
            FormatCategory::Image,
            FormatCategory::Document,
            FormatCategory::Audio,
            FormatCategory::Video,
            FormatCategory::Data,
            FormatCategory::Archive,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct FileFormat {
    pub id: &'static str,
    pub label: &'static str,
    pub extension: &'static str,
    pub category: FormatCategory,
    pub mime: &'static str,
}

impl FileFormat {
    pub fn display(&self) -> String {
        format!("{} (.{})", self.label, self.extension)
    }
}

pub struct FormatRegistry {
    pub formats: Vec<FileFormat>,
    compatibility: HashMap<&'static str, Vec<&'static str>>,
}

impl FormatRegistry {
    pub fn formats_in_category(&self, cat: &FormatCategory) -> Vec<&FileFormat> {
        self.formats.iter().filter(|f| &f.category == cat).collect()
    }

    pub fn targets_for(&self, source_id: &str) -> Vec<&FileFormat> {
        let ids = self.compatibility.get(source_id).cloned().unwrap_or_default();
        self.formats.iter().filter(|f| ids.contains(&f.id)).collect()
    }

    pub fn find(&self, id: &str) -> Option<&FileFormat> {
        self.formats.iter().find(|f| f.id == id)
    }
}

pub static REGISTRY: std::sync::OnceLock<FormatRegistry> = std::sync::OnceLock::new();

pub fn init_registry() -> FormatRegistry {
    let formats = vec![
        FileFormat { id: "png", label: "PNG", extension: "png", category: FormatCategory::Image, mime: "image/png" },
        FileFormat { id: "jpg", label: "JPEG", extension: "jpg", category: FormatCategory::Image, mime: "image/jpeg" },
        FileFormat { id: "webp", label: "WebP", extension: "webp", category: FormatCategory::Image, mime: "image/webp" },
        FileFormat { id: "gif", label: "GIF", extension: "gif", category: FormatCategory::Image, mime: "image/gif" },
        FileFormat { id: "bmp", label: "BMP", extension: "bmp", category: FormatCategory::Image, mime: "image/bmp" },
        FileFormat { id: "tiff", label: "TIFF", extension: "tiff", category: FormatCategory::Image, mime: "image/tiff" },
        FileFormat { id: "ico", label: "ICO", extension: "ico", category: FormatCategory::Image, mime: "image/x-icon" },

        FileFormat { id: "pdf", label: "PDF", extension: "pdf", category: FormatCategory::Document, mime: "application/pdf" },
        FileFormat { id: "txt", label: "Plain Text", extension: "txt", category: FormatCategory::Document, mime: "text/plain" },
        FileFormat { id: "md", label: "Markdown", extension: "md", category: FormatCategory::Document, mime: "text/markdown" },
        FileFormat { id: "html", label: "HTML", extension: "html", category: FormatCategory::Document, mime: "text/html" },
        FileFormat { id: "epub", label: "EPUB", extension: "epub", category: FormatCategory::Document, mime: "application/epub+zip" },

        FileFormat { id: "mp3", label: "MP3", extension: "mp3", category: FormatCategory::Audio, mime: "audio/mpeg" },
        FileFormat { id: "wav", label: "WAV", extension: "wav", category: FormatCategory::Audio, mime: "audio/wav" },
        FileFormat { id: "flac", label: "FLAC", extension: "flac", category: FormatCategory::Audio, mime: "audio/flac" },
        FileFormat { id: "ogg", label: "OGG Vorbis", extension: "ogg", category: FormatCategory::Audio, mime: "audio/ogg" },
        FileFormat { id: "aac", label: "AAC", extension: "aac", category: FormatCategory::Audio, mime: "audio/aac" },
        FileFormat { id: "m4a", label: "M4A", extension: "m4a", category: FormatCategory::Audio, mime: "audio/mp4" },

        FileFormat { id: "mp4", label: "MP4", extension: "mp4", category: FormatCategory::Video, mime: "video/mp4" },
        FileFormat { id: "mkv", label: "MKV", extension: "mkv", category: FormatCategory::Video, mime: "video/x-matroska" },
        FileFormat { id: "webm", label: "WebM", extension: "webm", category: FormatCategory::Video, mime: "video/webm" },
        FileFormat { id: "avi", label: "AVI", extension: "avi", category: FormatCategory::Video, mime: "video/x-msvideo" },
        FileFormat { id: "mov", label: "QuickTime", extension: "mov", category: FormatCategory::Video, mime: "video/quicktime" },

        FileFormat { id: "json", label: "JSON", extension: "json", category: FormatCategory::Data, mime: "application/json" },
        FileFormat { id: "csv", label: "CSV", extension: "csv", category: FormatCategory::Data, mime: "text/csv" },
        FileFormat { id: "xml", label: "XML", extension: "xml", category: FormatCategory::Data, mime: "application/xml" },
        FileFormat { id: "yaml", label: "YAML", extension: "yaml", category: FormatCategory::Data, mime: "application/x-yaml" },
        FileFormat { id: "toml", label: "TOML", extension: "toml", category: FormatCategory::Data, mime: "application/toml" },
        FileFormat { id: "tsv", label: "TSV", extension: "tsv", category: FormatCategory::Data, mime: "text/tab-separated-values" },

        FileFormat { id: "zip", label: "ZIP", extension: "zip", category: FormatCategory::Archive, mime: "application/zip" },
        FileFormat { id: "tar", label: "TAR", extension: "tar", category: FormatCategory::Archive, mime: "application/x-tar" },
        FileFormat { id: "tar_gz", label: "TAR.GZ", extension: "tar.gz", category: FormatCategory::Archive, mime: "application/gzip" },
    ];

    let mut compat: HashMap<&'static str, Vec<&'static str>> = HashMap::new();

    compat.insert("png", vec!["jpg","webp","gif","bmp","tiff","ico","pdf"]);
    compat.insert("jpg", vec!["png","webp","gif","bmp","tiff","ico","pdf"]);
    compat.insert("webp", vec!["png","jpg","gif","bmp","tiff"]);
    compat.insert("gif", vec!["png","jpg","webp","mp4","webm"]);
    compat.insert("bmp", vec!["png","jpg","webp","tiff"]);
    compat.insert("tiff", vec!["png","jpg","pdf","bmp"]);
    compat.insert("ico", vec!["png","bmp"]);

    compat.insert("txt", vec!["md","html"]);
    compat.insert("md", vec!["html","txt"]);
    compat.insert("html", vec!["txt"]);
    compat.insert("epub", vec!["txt","html"]);

    compat.insert("mp3", vec!["wav","flac","ogg","aac","m4a"]);
    compat.insert("wav", vec!["mp3","flac","ogg","aac"]);
    compat.insert("flac", vec!["mp3","wav","ogg","aac"]);
    compat.insert("ogg", vec!["mp3","wav","flac","aac"]);
    compat.insert("aac", vec!["mp3","wav","flac","ogg","m4a"]);
    compat.insert("m4a", vec!["mp3","wav","aac","ogg"]);

    compat.insert("mp4", vec!["mkv","webm","avi","mov","mp3"]);
    compat.insert("mkv", vec!["mp4","webm","avi","mp3"]);
    compat.insert("webm", vec!["mp4","mkv"]);
    compat.insert("avi", vec!["mp4","mkv","mov"]);
    compat.insert("mov", vec!["mp4","mkv","avi"]);

    compat.insert("json", vec!["csv","xml","yaml","toml"]);
    compat.insert("csv", vec!["json","xml","tsv"]);
    compat.insert("xml", vec!["json","csv","yaml"]);
    compat.insert("yaml", vec!["json","toml","xml"]);
    compat.insert("toml", vec!["json","yaml"]);
    compat.insert("tsv", vec!["csv","json"]);

    compat.insert("zip", vec!["tar_gz"]);
    compat.insert("tar", vec!["zip"]);
    compat.insert("tar_gz", vec!["zip"]);

    FormatRegistry { formats, compatibility: compat }
}