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
        FileFormat { id: "svg", label: "SVG", extension: "svg", category: FormatCategory::Image, mime: "image/svg+xml" },

        FileFormat { id: "pdf", label: "PDF", extension: "pdf", category: FormatCategory::Document, mime: "application/pdf" },
        FileFormat { id: "txt", label: "Plain Text", extension: "txt", category: FormatCategory::Document, mime: "text/plain" },
        FileFormat { id: "md", label: "Markdown", extension: "md", category: FormatCategory::Document, mime: "text/markdown" },
        FileFormat { id: "html", label: "HTML", extension: "html", category: FormatCategory::Document, mime: "text/html" },
        FileFormat { id: "rtf", label: "RTF", extension: "rtf", category: FormatCategory::Document, mime: "application/rtf" },
        FileFormat { id: "epub", label: "EPUB", extension: "epub", category: FormatCategory::Document, mime: "application/epub+zip" },
        FileFormat { id: "docx", label: "Word DOCX", extension: "docx", category: FormatCategory::Document, mime: "application/vnd.openxmlformats-officedocument.wordprocessingml.document" },
        FileFormat { id: "odt", label: "OpenDocument Text", extension: "odt", category: FormatCategory::Document, mime: "application/vnd.oasis.opendocument.text" },
        FileFormat { id: "pptx", label: "PowerPoint", extension: "pptx", category: FormatCategory::Document, mime: "application/vnd.openxmlformats-officedocument.presentationml.presentation" },

        FileFormat { id: "mp3", label: "MP3", extension: "mp3", category: FormatCategory::Audio, mime: "audio/mpeg" },
        FileFormat { id: "wav", label: "WAV", extension: "wav", category: FormatCategory::Audio, mime: "audio/wav" },
        FileFormat { id: "flac", label: "FLAC", extension: "flac", category: FormatCategory::Audio, mime: "audio/flac" },
        FileFormat { id: "ogg", label: "OGG Vorbis", extension: "ogg", category: FormatCategory::Audio, mime: "audio/ogg" },
        FileFormat { id: "aac", label: "AAC", extension: "aac", category: FormatCategory::Audio, mime: "audio/aac" },
        FileFormat { id: "m4a", label: "M4A", extension: "m4a", category: FormatCategory::Audio, mime: "audio/mp4" },
        FileFormat { id: "opus", label: "Opus", extension: "opus", category: FormatCategory::Audio, mime: "audio/opus" },
        FileFormat { id: "wma", label: "WMA", extension: "wma", category: FormatCategory::Audio, mime: "audio/x-ms-wma" },
        FileFormat { id: "aiff", label: "AIFF", extension: "aiff", category: FormatCategory::Audio, mime: "audio/aiff" },

        FileFormat { id: "mp4", label: "MP4", extension: "mp4", category: FormatCategory::Video, mime: "video/mp4" },
        FileFormat { id: "mkv", label: "MKV", extension: "mkv", category: FormatCategory::Video, mime: "video/x-matroska" },
        FileFormat { id: "webm", label: "WebM", extension: "webm", category: FormatCategory::Video, mime: "video/webm" },
        FileFormat { id: "avi", label: "AVI", extension: "avi", category: FormatCategory::Video, mime: "video/x-msvideo" },
        FileFormat { id: "mov", label: "QuickTime MOV", extension: "mov", category: FormatCategory::Video, mime: "video/quicktime" },
        FileFormat { id: "flv", label: "Flash Video", extension: "flv", category: FormatCategory::Video, mime: "video/x-flv" },
        FileFormat { id: "wmv", label: "WMV", extension: "wmv", category: FormatCategory::Video, mime: "video/x-ms-wmv" },
        FileFormat { id: "m4v", label: "M4V", extension: "m4v", category: FormatCategory::Video, mime: "video/x-m4v" },
        FileFormat { id: "3gp", label: "3GP", extension: "3gp", category: FormatCategory::Video, mime: "video/3gpp" },
        FileFormat { id: "ts", label: "MPEG-TS", extension: "ts", category: FormatCategory::Video, mime: "video/mp2t" },

        FileFormat { id: "json", label: "JSON", extension: "json", category: FormatCategory::Data, mime: "application/json" },
        FileFormat { id: "csv", label: "CSV", extension: "csv", category: FormatCategory::Data, mime: "text/csv" },
        FileFormat { id: "tsv", label: "TSV", extension: "tsv", category: FormatCategory::Data, mime: "text/tab-separated-values" },
        FileFormat { id: "xml", label: "XML", extension: "xml", category: FormatCategory::Data, mime: "application/xml" },
        FileFormat { id: "yaml", label: "YAML", extension: "yaml", category: FormatCategory::Data, mime: "application/x-yaml" },
        FileFormat { id: "toml", label: "TOML", extension: "toml", category: FormatCategory::Data, mime: "application/toml" },
        FileFormat { id: "xlsx", label: "Excel XLSX", extension: "xlsx", category: FormatCategory::Data, mime: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" },
        FileFormat { id: "xls", label: "Excel XLS", extension: "xls", category: FormatCategory::Data, mime: "application/vnd.ms-excel" },
        FileFormat { id: "ods", label: "OpenDocument Spreadsheet", extension: "ods", category: FormatCategory::Data, mime: "application/vnd.oasis.opendocument.spreadsheet" },

        FileFormat { id: "zip", label: "ZIP", extension: "zip", category: FormatCategory::Archive, mime: "application/zip" },
        FileFormat { id: "tar", label: "TAR", extension: "tar", category: FormatCategory::Archive, mime: "application/x-tar" },
        FileFormat { id: "tar_gz", label: "TAR.GZ", extension: "tar.gz", category: FormatCategory::Archive, mime: "application/gzip" },
        FileFormat { id: "7z", label: "7-Zip", extension: "7z", category: FormatCategory::Archive, mime: "application/x-7z-compressed" },
    ];

    let mut compat: HashMap<&'static str, Vec<&'static str>> = HashMap::new();

    compat.insert("png", vec!["jpg","webp","gif","bmp","tiff","ico","pdf"]);
    compat.insert("jpg", vec!["png","webp","gif","bmp","tiff","ico","pdf"]);
    compat.insert("webp", vec!["png","jpg","gif","bmp","tiff"]);
    compat.insert("gif", vec!["png","jpg","webp","bmp","mp4"]);
    compat.insert("bmp", vec!["png","jpg","webp","tiff","ico"]);
    compat.insert("tiff", vec!["png","jpg","pdf","bmp"]);
    compat.insert("ico", vec!["png","bmp"]);
    compat.insert("svg", vec!["png","jpg","pdf","webp","bmp"]);

    compat.insert("pdf", vec!["txt","docx","html","md"]);
    compat.insert("txt", vec!["md","html","pdf","png","svg"]);
    compat.insert("md", vec!["html","txt","pdf"]);
    compat.insert("html", vec!["txt","md","pdf"]);
    compat.insert("rtf", vec!["txt","pdf","docx"]);
    compat.insert("epub", vec!["txt","html"]);
    compat.insert("docx", vec!["pdf","txt","html","md","odt","rtf"]);
    compat.insert("odt", vec!["pdf","txt","html","docx"]);
    compat.insert("pptx", vec!["pdf","html"]);

    compat.insert("mp3", vec!["wav","flac","ogg","aac","m4a","opus"]);
    compat.insert("wav", vec!["mp3","flac","ogg","aac","opus","m4a"]);
    compat.insert("flac", vec!["mp3","wav","ogg","aac"]);
    compat.insert("ogg", vec!["mp3","wav","flac","aac"]);
    compat.insert("aac", vec!["mp3","wav","flac","ogg","m4a"]);
    compat.insert("m4a", vec!["mp3","wav","aac","ogg","flac"]);
    compat.insert("opus", vec!["mp3","wav","ogg","aac"]);
    compat.insert("wma", vec!["mp3","wav","flac","aac"]);
    compat.insert("aiff", vec!["mp3","wav","flac","aac"]);

    compat.insert("mp4", vec!["mkv","webm","avi","mov","mp3","wav","gif"]);
    compat.insert("mkv", vec!["mp4","webm","avi","mov","mp3"]);
    compat.insert("webm", vec!["mp4","mkv","gif"]);
    compat.insert("avi", vec!["mp4","mkv","mov","mp3"]);
    compat.insert("mov", vec!["mp4","mkv","avi","mp3"]);
    compat.insert("flv", vec!["mp4","mkv","avi"]);
    compat.insert("wmv", vec!["mp4","mkv","avi"]);
    compat.insert("m4v", vec!["mp4","mkv","avi"]);
    compat.insert("3gp", vec!["mp4","mkv"]);
    compat.insert("ts", vec!["mp4","mkv"]);

    compat.insert("json", vec!["csv","xml","yaml","toml","xlsx"]);
    compat.insert("csv", vec!["json","xml","tsv","xlsx","png","svg"]);
    compat.insert("tsv", vec!["csv","json"]);
    compat.insert("xml", vec!["json","csv","yaml"]);
    compat.insert("yaml", vec!["json","toml","xml"]);
    compat.insert("toml", vec!["json","yaml"]);
    compat.insert("xlsx", vec!["csv","json"]);
    compat.insert("xls", vec!["xlsx","csv","json"]);
    compat.insert("ods", vec!["xlsx","csv","json"]);

    compat.insert("zip", vec!["tar_gz"]);
    compat.insert("tar", vec!["zip","tar_gz"]);
    compat.insert("tar_gz", vec!["zip"]);
    compat.insert("7z", vec!["zip","tar_gz"]);

    FormatRegistry { formats, compatibility: compat }
}