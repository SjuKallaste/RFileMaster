use std::path::{Path, PathBuf};
use std::fs;
use std::io::{Read, Write};
use image::ImageFormat;
use printpdf::{Mm, PdfDocument, Image as PdfImage, ImageTransform, ImageXObject, ColorSpace, ColorBits};
use crate::conversion::external;

pub fn convert(inputs: &[PathBuf], source: &str, target: &str, output_path: &Path, merge: bool) -> Result<(), String> {
    if inputs.is_empty() {
        return Err("No input files provided".to_string());
    }
    match (source, target) {
        (s, "pdf") if is_raster_image(s) => images_to_pdf(inputs, output_path),
        ("svg", "pdf") => batch_or_single(inputs, output_path, merge, svg_to_pdf),
        ("svg", t) if is_raster_image(t) => batch_or_single(inputs, output_path, merge, |i, o| svg_to_raster(i, o, t)),
        (s, t) if is_raster_image(s) && is_raster_image(t) => batch_or_single(inputs, output_path, merge, |i, o| image_to_image(i, o, t)),
        ("pdf", "txt") => batch_or_single(inputs, output_path, merge, pdf_to_txt),
        ("json", "csv") => batch_or_single(inputs, output_path, merge, json_to_csv),
        ("csv", "json") => {
            if merge && inputs.len() > 1 { csvs_to_json_merged(inputs, output_path) }
            else { batch_or_single(inputs, output_path, merge, csv_to_json) }
        }
        ("json", "yaml") => batch_or_single(inputs, output_path, merge, json_to_yaml),
        ("yaml", "json") => batch_or_single(inputs, output_path, merge, yaml_to_json),
        ("json", "toml") => batch_or_single(inputs, output_path, merge, json_to_toml),
        ("toml", "json") => batch_or_single(inputs, output_path, merge, toml_to_json),
        ("yaml", "toml") => batch_or_single(inputs, output_path, merge, yaml_to_toml),
        ("toml", "yaml") => batch_or_single(inputs, output_path, merge, toml_to_yaml),
        ("csv", "tsv") => batch_or_single(inputs, output_path, merge, csv_to_tsv),
        ("tsv", "csv") => batch_or_single(inputs, output_path, merge, tsv_to_csv),
        ("xml", "json") => batch_or_single(inputs, output_path, merge, xml_to_json),
        ("json", "xml") => batch_or_single(inputs, output_path, merge, json_to_xml),
        ("xml", "csv") => batch_or_single(inputs, output_path, merge, xml_to_csv),
        ("xml", "yaml") => batch_or_single(inputs, output_path, merge, xml_to_yaml),
        ("yaml", "xml") => batch_or_single(inputs, output_path, merge, yaml_to_xml),
        ("csv", "xlsx") => batch_or_single(inputs, output_path, merge, csv_to_xlsx),
        ("json", "xlsx") => batch_or_single(inputs, output_path, merge, json_to_xlsx),
        ("xlsx", "csv") => batch_or_single(inputs, output_path, merge, xlsx_to_csv),
        ("xlsx", "json") => batch_or_single(inputs, output_path, merge, xlsx_to_json),
        ("xls", "csv") => batch_or_single(inputs, output_path, merge, xls_to_csv),
        ("xls", "json") => batch_or_single(inputs, output_path, merge, xls_to_json),
        ("xls", "xlsx") => batch_or_single(inputs, output_path, merge, xls_to_xlsx),
        ("md", "html") => {
            if merge && inputs.len() > 1 { mds_to_html_merged(inputs, output_path) }
            else { batch_or_single(inputs, output_path, merge, md_to_html) }
        }
        ("html", "txt") => batch_or_single(inputs, output_path, merge, html_to_txt),
        ("html", "md") => batch_or_single(inputs, output_path, merge, html_to_md),
        ("txt", "md") => batch_or_single(inputs, output_path, merge, txt_to_md),
        ("txt", "pdf") => batch_or_single(inputs, output_path, merge, txt_to_pdf),
        ("rtf", "txt") => batch_or_single(inputs, output_path, merge, rtf_to_txt),
        ("epub", "txt") => batch_or_single(inputs, output_path, merge, epub_to_txt),
        ("epub", "html") => batch_or_single(inputs, output_path, merge, epub_to_html),
        ("zip", "tar_gz") => batch_or_single(inputs, output_path, merge, zip_to_tar_gz),
        ("tar_gz", "zip") | ("tar", "zip") => batch_or_single(inputs, output_path, merge, tar_gz_to_zip),
        ("tar", "tar_gz") => batch_or_single(inputs, output_path, merge, tar_to_tar_gz),
        ("7z", "zip") => batch_or_single(inputs, output_path, merge, sevenz_to_zip),
        ("7z", "tar_gz") => batch_or_single(inputs, output_path, merge, sevenz_to_tar_gz),

        ("docx", "txt") => batch_or_single(inputs, output_path, merge, docx_to_txt),
        ("docx", "md") => batch_or_single(inputs, output_path, merge, docx_to_md),
        ("ods", "csv") => batch_or_single(inputs, output_path, merge, xlsx_to_csv),
        ("ods", "json") => batch_or_single(inputs, output_path, merge, xlsx_to_json),
        ("ods", "xlsx") => batch_or_single(inputs, output_path, merge, xls_to_xlsx),

        ("txt", "png") => batch_or_single(inputs, output_path, merge, |i, o| txt_to_qr(i, o, "png")),
        ("txt", "svg") => batch_or_single(inputs, output_path, merge, |i, o| txt_to_qr(i, o, "svg")),
        ("csv", "png") => batch_or_single(inputs, output_path, merge, |i, o| csv_to_chart(i, o, "png")),
        ("csv", "svg") => batch_or_single(inputs, output_path, merge, |i, o| csv_to_chart(i, o, "svg")),

        (s, t) if is_audio(s) && is_audio(t) => batch_or_single(inputs, output_path, merge, |i, o| external::ffmpeg_audio(i, o, t)),
        (s, t) if is_video(s) && (is_video(t) || is_audio(t)) => batch_or_single(inputs, output_path, merge, |i, o| external::ffmpeg_video(i, o, t)),
        (s, "gif") if is_video(s) => batch_or_single(inputs, output_path, merge, |i, o| external::ffmpeg_video(i, o, "gif")),

        ("docx", t) if matches!(t, "pdf"|"txt"|"html"|"md"|"odt"|"rtf") => batch_or_single(inputs, output_path, merge, |i, o| docx_via_libreoffice_or_pandoc(i, o, t)),
        ("odt", t) if matches!(t, "pdf"|"txt"|"html"|"docx") => batch_or_single(inputs, output_path, merge, |i, o| external::libreoffice_convert(i, t, o.parent().unwrap_or(Path::new(".")))),
        ("pdf", t) if matches!(t, "docx"|"html"|"md") => batch_or_single(inputs, output_path, merge, |i, o| external::pandoc(i, o, &[])),
        (s, "pdf") if matches!(s, "docx"|"odt"|"html"|"md") => batch_or_single(inputs, output_path, merge, |i, o| doc_to_pdf(i, o, s)),
        ("pptx", "pdf") => batch_or_single(inputs, output_path, merge, |i, o| external::libreoffice_convert(i, "pdf", o.parent().unwrap_or(Path::new(".")))),
        ("pptx", "html") => batch_or_single(inputs, output_path, merge, |i, o| external::pandoc(i, o, &[])),
        _ => Err(format!("Conversion from {} to {} requires an external tool and is not supported natively", source, target)),
    }
}

fn batch_or_single(inputs: &[PathBuf], output_path: &Path, _merge: bool, f: impl Fn(&Path, &Path) -> Result<(), String>) -> Result<(), String> {
    if inputs.len() == 1 {
        return f(&inputs[0], output_path);
    }
    let out_dir = output_path.parent().unwrap_or(Path::new("."));
    let ext = output_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    for input in inputs {
        let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
        f(input, &out_dir.join(format!("{}.{}", stem, ext)))?;
    }
    Ok(())
}

pub fn supports_merge(source: &str, target: &str) -> bool {
    (is_raster_image(source) && target == "pdf")
        || (source == "md" && target == "html")
        || (source == "csv" && target == "json")
}

fn is_raster_image(fmt: &str) -> bool {
    matches!(fmt, "png"|"jpg"|"jpeg"|"webp"|"bmp"|"tiff"|"gif"|"ico")
}

fn is_audio(fmt: &str) -> bool {
    matches!(fmt, "mp3"|"wav"|"flac"|"ogg"|"aac"|"m4a"|"opus"|"wma"|"aiff")
}

fn is_video(fmt: &str) -> bool {
    matches!(fmt, "mp4"|"mkv"|"webm"|"avi"|"mov"|"flv"|"wmv"|"m4v"|"3gp"|"ts")
}

fn docx_via_libreoffice_or_pandoc(input: &Path, output: &Path, target: &str) -> Result<(), String> {
    if external::ExternalTool::LibreOffice.find().is_some() {
        let out_dir = output.parent().unwrap_or(Path::new("."));
        external::libreoffice_convert(input, target, out_dir)
    } else {
        external::pandoc(input, output, &[])
    }
}

fn doc_to_pdf(input: &Path, output: &Path, source: &str) -> Result<(), String> {
    if source == "md" || source == "html" {
        if external::ExternalTool::Pandoc.find().is_some() {
            return external::pandoc(input, output, &["--pdf-engine=xelatex"]);
        }
    }
    let out_dir = output.parent().unwrap_or(Path::new("."));
    external::libreoffice_convert(input, "pdf", out_dir)
}

fn ext_to_image_format(ext: &str) -> Option<ImageFormat> {
    match ext {
        "png" => Some(ImageFormat::Png),
        "jpg"|"jpeg" => Some(ImageFormat::Jpeg),
        "bmp" => Some(ImageFormat::Bmp),
        "tiff" => Some(ImageFormat::Tiff),
        "gif" => Some(ImageFormat::Gif),
        "webp" => Some(ImageFormat::WebP),
        "ico" => Some(ImageFormat::Ico),
        _ => None,
    }
}

fn image_to_image(input: &Path, output: &Path, target_ext: &str) -> Result<(), String> {
    let fmt = ext_to_image_format(target_ext).ok_or_else(|| format!("Unsupported image format: {}", target_ext))?;
    image::open(input).map_err(|e| e.to_string())?.save_with_format(output, fmt).map_err(|e| e.to_string())
}

fn svg_to_raster(input: &Path, output: &Path, target_ext: &str) -> Result<(), String> {
    let svg = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let mut opts = svg2pdf::usvg::Options::default();
    opts.fontdb_mut().load_system_fonts();
    let tree = svg2pdf::usvg::Tree::from_str(&svg, &opts).map_err(|e| e.to_string())?;
    let size = tree.size();
    let w = size.width() as u32;
    let h = size.height() as u32;
    let mut pixmap = tiny_skia::Pixmap::new(w, h).ok_or("Failed to create pixmap")?;
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
    let img = image::RgbaImage::from_raw(w, h, pixmap.take()).ok_or("Failed to convert pixmap")?;
    let fmt = ext_to_image_format(target_ext).ok_or_else(|| format!("Unsupported format: {}", target_ext))?;
    image::DynamicImage::ImageRgba8(img).save_with_format(output, fmt).map_err(|e| e.to_string())
}

fn svg_to_pdf(input: &Path, output: &Path) -> Result<(), String> {
    let svg = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let mut opts = svg2pdf::usvg::Options::default();
    opts.fontdb_mut().load_system_fonts();
    let tree = svg2pdf::usvg::Tree::from_str(&svg, &opts).map_err(|e| e.to_string())?;
    let pdf = svg2pdf::to_pdf(&tree, svg2pdf::ConversionOptions::default(), svg2pdf::PageOptions::default())
        .map_err(|e| e.to_string())?;
    fs::write(output, pdf).map_err(|e| e.to_string())
}

fn add_img_to_pdf_layer(img: image::DynamicImage, layer: printpdf::PdfLayerReference) {
    let (w, h) = (img.width(), img.height());
    let raw = img.to_rgb8().into_raw();
    PdfImage::from(ImageXObject {
        width: printpdf::Px(w as usize),
        height: printpdf::Px(h as usize),
        color_space: ColorSpace::Rgb,
        bits_per_component: ColorBits::Bit8,
        interpolate: true,
        image_data: raw,
        image_filter: None,
        smask: None,
        clipping_bbox: None,
    }).add_to_layer(layer, ImageTransform {
        translate_x: Some(Mm(0.0)),
        translate_y: Some(Mm(0.0)),
        scale_x: None,
        scale_y: None,
        dpi: Some(150.0),
        ..Default::default()
    });
}

fn images_to_pdf(inputs: &[PathBuf], output: &Path) -> Result<(), String> {
    let mm_per_px = 25.4 / 150.0_f32;
    let first = image::open(&inputs[0]).map_err(|e| e.to_string())?;
    let (fw, fh) = (first.width() as f32, first.height() as f32);
    let (doc, first_page, first_layer) = PdfDocument::new("RFileMaster Export", Mm(fw * mm_per_px), Mm(fh * mm_per_px), "Layer 1");
    add_img_to_pdf_layer(first, doc.get_page(first_page).get_layer(first_layer));
    for input in inputs.iter().skip(1) {
        let img = image::open(input).map_err(|e| e.to_string())?;
        let (iw, ih) = (img.width() as f32, img.height() as f32);
        let (page_idx, layer_idx) = doc.add_page(Mm(iw * mm_per_px), Mm(ih * mm_per_px), "Layer 1");
        add_img_to_pdf_layer(img, doc.get_page(page_idx).get_layer(layer_idx));
    }
    fs::write(output, doc.save_to_bytes().map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn pdf_to_txt(input: &Path, output: &Path) -> Result<(), String> {
    let doc = lopdf::Document::load(input).map_err(|e| e.to_string())?;
    let mut text = String::new();
    let pages: Vec<u32> = doc.get_pages().keys().cloned().collect();
    for page_num in pages {
        if let Ok(page_text) = doc.extract_text(&[page_num]) {
            text.push_str(&page_text);
            text.push('\n');
        }
    }
    fs::write(output, text.trim()).map_err(|e| e.to_string())
}

fn csv_to_xlsx(input: &Path, output: &Path) -> Result<(), String> {
    let mut rdr = csv::Reader::from_path(input).map_err(|e| e.to_string())?;
    let headers: Vec<String> = rdr.headers().map_err(|e| e.to_string())?.iter().map(|s| s.to_string()).collect();
    let mut workbook = rust_xlsxwriter::Workbook::new();
    let sheet = workbook.add_worksheet();
    for (col, h) in headers.iter().enumerate() {
        sheet.write_string(0, col as u16, h).map_err(|e| e.to_string())?;
    }
    for (row, result) in rdr.records().enumerate() {
        let record = result.map_err(|e| e.to_string())?;
        for (col, val) in record.iter().enumerate() {
            sheet.write_string((row + 1) as u32, col as u16, val).map_err(|e| e.to_string())?;
        }
    }
    workbook.save(output).map_err(|e| e.to_string())
}

fn json_to_xlsx(input: &Path, output: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let rows = value.as_array().ok_or("JSON root must be an array of objects")?;
    if rows.is_empty() {
        let mut wb = rust_xlsxwriter::Workbook::new();
        wb.add_worksheet();
        return wb.save(output).map_err(|e| e.to_string());
    }
    let headers: Vec<String> = rows[0].as_object().ok_or("Elements must be objects")?.keys().cloned().collect();
    let mut workbook = rust_xlsxwriter::Workbook::new();
    let sheet = workbook.add_worksheet();
    for (col, h) in headers.iter().enumerate() {
        sheet.write_string(0, col as u16, h).map_err(|e| e.to_string())?;
    }
    for (row, item) in rows.iter().enumerate() {
        if let Some(obj) = item.as_object() {
            for (col, h) in headers.iter().enumerate() {
                let val = obj.get(h).map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                }).unwrap_or_default();
                sheet.write_string((row + 1) as u32, col as u16, &val).map_err(|e| e.to_string())?;
            }
        }
    }
    workbook.save(output).map_err(|e| e.to_string())
}

fn xlsx_to_csv(input: &Path, output: &Path) -> Result<(), String> {
    use calamine::Reader;
    let mut wb = calamine::open_workbook_auto(input).map_err(|e| e.to_string())?;
    let name = wb.sheet_names().first().ok_or("No sheets found")?.clone();
    let range = wb.worksheet_range(&name).map_err(|e| e.to_string())?;
    let mut wtr = csv::Writer::from_path(output).map_err(|e| e.to_string())?;
    for row in range.rows() {
        let record: Vec<String> = row.iter().map(|c| c.to_string()).collect();
        wtr.write_record(&record).map_err(|e| e.to_string())?;
    }
    wtr.flush().map_err(|e| e.to_string())
}

fn xlsx_to_json(input: &Path, output: &Path) -> Result<(), String> {
    use calamine::Reader;
    let mut wb = calamine::open_workbook_auto(input).map_err(|e| e.to_string())?;
    let name = wb.sheet_names().first().ok_or("No sheets found")?.clone();
    let range = wb.worksheet_range(&name).map_err(|e| e.to_string())?;
    let mut rows = range.rows();
    let headers: Vec<String> = match rows.next() {
        Some(h) => h.iter().map(|c| c.to_string()).collect(),
        None => return fs::write(output, "[]").map_err(|e| e.to_string()),
    };
    let records: Vec<serde_json::Value> = rows.map(|row| {
        let obj: serde_json::Map<String, serde_json::Value> = headers.iter().zip(row.iter())
            .map(|(h, v)| (h.clone(), serde_json::Value::String(v.to_string())))
            .collect();
        serde_json::Value::Object(obj)
    }).collect();
    fs::write(output, serde_json::to_string_pretty(&records).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn xls_to_csv(input: &Path, output: &Path) -> Result<(), String> {
    xlsx_to_csv(input, output)
}

fn xls_to_json(input: &Path, output: &Path) -> Result<(), String> {
    xlsx_to_json(input, output)
}

fn xls_to_xlsx(input: &Path, output: &Path) -> Result<(), String> {
    use calamine::Reader;
    let mut wb = calamine::open_workbook_auto(input).map_err(|e| e.to_string())?;
    let name = wb.sheet_names().first().ok_or("No sheets found")?.clone();
    let range = wb.worksheet_range(&name).map_err(|e| e.to_string())?;
    let mut workbook = rust_xlsxwriter::Workbook::new();
    let sheet = workbook.add_worksheet();
    for (row_idx, row) in range.rows().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            sheet.write_string(row_idx as u32, col_idx as u16, &cell.to_string()).map_err(|e| e.to_string())?;
        }
    }
    workbook.save(output).map_err(|e| e.to_string())
}

fn json_to_csv(input: &Path, output: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let rows = value.as_array().ok_or("JSON root must be an array of objects")?;
    if rows.is_empty() {
        return fs::write(output, "").map_err(|e| e.to_string());
    }
    let headers: Vec<String> = rows[0].as_object().ok_or("Each element must be an object")?.keys().cloned().collect();
    let mut wtr = csv::Writer::from_path(output).map_err(|e| e.to_string())?;
    wtr.write_record(&headers).map_err(|e| e.to_string())?;
    for row in rows {
        let obj = row.as_object().ok_or("Each element must be an object")?;
        let record: Vec<String> = headers.iter().map(|h| {
            obj.get(h).map(|v| match v {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            }).unwrap_or_default()
        }).collect();
        wtr.write_record(&record).map_err(|e| e.to_string())?;
    }
    wtr.flush().map_err(|e| e.to_string())
}

fn csv_to_json(input: &Path, output: &Path) -> Result<(), String> {
    let mut rdr = csv::Reader::from_path(input).map_err(|e| e.to_string())?;
    let headers: Vec<String> = rdr.headers().map_err(|e| e.to_string())?.iter().map(|s| s.to_string()).collect();
    let mut records: Vec<serde_json::Value> = Vec::new();
    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        let obj: serde_json::Map<String, serde_json::Value> = headers.iter().zip(record.iter())
            .map(|(h, v)| (h.clone(), serde_json::Value::String(v.to_string())))
            .collect();
        records.push(serde_json::Value::Object(obj));
    }
    fs::write(output, serde_json::to_string_pretty(&records).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn csvs_to_json_merged(inputs: &[PathBuf], output: &Path) -> Result<(), String> {
    let mut all: Vec<serde_json::Value> = Vec::new();
    for input in inputs {
        let mut rdr = csv::Reader::from_path(input).map_err(|e| e.to_string())?;
        let headers: Vec<String> = rdr.headers().map_err(|e| e.to_string())?.iter().map(|s| s.to_string()).collect();
        for result in rdr.records() {
            let record = result.map_err(|e| e.to_string())?;
            let obj: serde_json::Map<String, serde_json::Value> = headers.iter().zip(record.iter())
                .map(|(h, v)| (h.clone(), serde_json::Value::String(v.to_string())))
                .collect();
            all.push(serde_json::Value::Object(obj));
        }
    }
    fs::write(output, serde_json::to_string_pretty(&all).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn json_to_yaml(input: &Path, output: &Path) -> Result<(), String> {
    let value: serde_json::Value = serde_json::from_str(&fs::read_to_string(input).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    fs::write(output, serde_yaml::to_string(&value).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn yaml_to_json(input: &Path, output: &Path) -> Result<(), String> {
    let value: serde_yaml::Value = serde_yaml::from_str(&fs::read_to_string(input).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    fs::write(output, serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn json_to_toml(input: &Path, output: &Path) -> Result<(), String> {
    let value: toml::Value = serde_json::from_str::<serde_json::Value>(&fs::read_to_string(input).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
        .and_then(|v| toml::Value::try_from(v).map_err(|e| e.to_string()))?;
    fs::write(output, toml::to_string_pretty(&value).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn toml_to_json(input: &Path, output: &Path) -> Result<(), String> {
    let value: toml::Value = toml::from_str(&fs::read_to_string(input).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    fs::write(output, serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn yaml_to_toml(input: &Path, output: &Path) -> Result<(), String> {
    let value: serde_yaml::Value = serde_yaml::from_str(&fs::read_to_string(input).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let json_val: serde_json::Value = serde_json::to_value(&value).map_err(|e| e.to_string())?;
    let toml_val: toml::Value = toml::Value::try_from(json_val).map_err(|e| e.to_string())?;
    fs::write(output, toml::to_string_pretty(&toml_val).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn toml_to_yaml(input: &Path, output: &Path) -> Result<(), String> {
    let value: toml::Value = toml::from_str(&fs::read_to_string(input).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    fs::write(output, serde_yaml::to_string(&value).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn csv_to_tsv(input: &Path, output: &Path) -> Result<(), String> {
    let mut rdr = csv::Reader::from_path(input).map_err(|e| e.to_string())?;
    let mut wtr = csv::WriterBuilder::new().delimiter(b'\t').from_path(output).map_err(|e| e.to_string())?;
    let headers = rdr.headers().map_err(|e| e.to_string())?.clone();
    wtr.write_record(&headers).map_err(|e| e.to_string())?;
    for result in rdr.records() {
        wtr.write_record(&result.map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    }
    wtr.flush().map_err(|e| e.to_string())
}

fn tsv_to_csv(input: &Path, output: &Path) -> Result<(), String> {
    let mut rdr = csv::ReaderBuilder::new().delimiter(b'\t').from_path(input).map_err(|e| e.to_string())?;
    let mut wtr = csv::Writer::from_path(output).map_err(|e| e.to_string())?;
    let headers = rdr.headers().map_err(|e| e.to_string())?.clone();
    wtr.write_record(&headers).map_err(|e| e.to_string())?;
    for result in rdr.records() {
        wtr.write_record(&result.map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    }
    wtr.flush().map_err(|e| e.to_string())
}

fn xml_text_to_value(text: &str) -> Result<serde_json::Value, String> {
    let mut reader = quick_xml::Reader::from_str(text);
    reader.config_mut().trim_text(true);
    let mut stack: Vec<(String, Vec<serde_json::Value>)> = Vec::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(e)) => {
                stack.push((String::from_utf8_lossy(e.name().as_ref()).to_string(), Vec::new()));
            }
            Ok(quick_xml::events::Event::End(_)) => {
                if let Some((name, children)) = stack.pop() {
                    let val = if children.is_empty() { serde_json::Value::Null }
                    else if children.len() == 1 { children.into_iter().next().unwrap() }
                    else { serde_json::Value::Array(children) };
                    if let Some(parent) = stack.last_mut() {
                        parent.1.push(serde_json::json!({ &name: val }));
                    } else {
                        return Ok(serde_json::json!({ &name: val }));
                    }
                }
            }
            Ok(quick_xml::events::Event::Text(e)) => {
                let t = e.unescape().map_err(|e| e.to_string())?;
                if !t.trim().is_empty() {
                    if let Some(parent) = stack.last_mut() {
                        parent.1.push(serde_json::Value::String(t.to_string()));
                    }
                }
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(e) => return Err(e.to_string()),
            _ => {}
        }
        buf.clear();
    }
    Err("Empty or invalid XML".to_string())
}

fn value_to_xml(value: &serde_json::Value, tag: &str) -> String {
    match value {
        serde_json::Value::Object(map) => {
            let inner: String = map.iter().map(|(k, v)| value_to_xml(v, k)).collect();
            format!("<{}>{}</{}>", tag, inner, tag)
        }
        serde_json::Value::Array(arr) => arr.iter().map(|v| value_to_xml(v, tag)).collect(),
        serde_json::Value::Null => format!("<{}/>", tag),
        other => format!("<{}>{}</{}>", tag, other.to_string().trim_matches('"'), tag),
    }
}

fn xml_to_json(input: &Path, output: &Path) -> Result<(), String> {
    let value = xml_text_to_value(&fs::read_to_string(input).map_err(|e| e.to_string())?)?;
    fs::write(output, serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn json_to_xml(input: &Path, output: &Path) -> Result<(), String> {
    let value: serde_json::Value = serde_json::from_str(&fs::read_to_string(input).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    fs::write(output, format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", value_to_xml(&value, "root"))).map_err(|e| e.to_string())
}

fn xml_to_csv(input: &Path, output: &Path) -> Result<(), String> {
    let value = xml_text_to_value(&fs::read_to_string(input).map_err(|e| e.to_string())?)?;
    let tmp = std::env::temp_dir().join("rfilemaster_xml_tmp.json");
    fs::write(&tmp, serde_json::to_string(&value).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let result = json_to_csv(&tmp, output);
    let _ = fs::remove_file(tmp);
    result
}

fn xml_to_yaml(input: &Path, output: &Path) -> Result<(), String> {
    let value = xml_text_to_value(&fs::read_to_string(input).map_err(|e| e.to_string())?)?;
    fs::write(output, serde_yaml::to_string(&value).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn yaml_to_xml(input: &Path, output: &Path) -> Result<(), String> {
    let value: serde_yaml::Value = serde_yaml::from_str(&fs::read_to_string(input).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let json_val: serde_json::Value = serde_json::to_value(&value).map_err(|e| e.to_string())?;
    fs::write(output, format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", value_to_xml(&json_val, "root"))).map_err(|e| e.to_string())
}

fn md_to_html(input: &Path, output: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let parser = pulldown_cmark::Parser::new(&text);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    fs::write(output, format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><style>body{{font-family:sans-serif;max-width:800px;margin:40px auto;line-height:1.6;padding:0 20px}}</style></head><body>{}</body></html>",
        html
    )).map_err(|e| e.to_string())
}

fn mds_to_html_merged(inputs: &[PathBuf], output: &Path) -> Result<(), String> {
    let mut combined = String::new();
    for (i, input) in inputs.iter().enumerate() {
        if i > 0 { combined.push_str("\n\n<hr>\n\n"); }
        combined.push_str(&fs::read_to_string(input).map_err(|e| e.to_string())?);
    }
    let parser = pulldown_cmark::Parser::new(&combined);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    fs::write(output, format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><style>body{{font-family:sans-serif;max-width:800px;margin:40px auto;line-height:1.6;padding:0 20px}}</style></head><body>{}</body></html>",
        html
    )).map_err(|e| e.to_string())
}

fn strip_html(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => { in_tag = true; }
            '>' => { in_tag = false; out.push(' '); }
            c if !in_tag => { out.push(c); }
            _ => {}
        }
    }
    out.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect::<Vec<_>>().join("\n")
}

fn html_to_txt(input: &Path, output: &Path) -> Result<(), String> {
    fs::write(output, strip_html(&fs::read_to_string(input).map_err(|e| e.to_string())?)).map_err(|e| e.to_string())
}

fn html_to_md(input: &Path, output: &Path) -> Result<(), String> {
    let text = strip_html(&fs::read_to_string(input).map_err(|e| e.to_string())?);
    fs::write(output, text).map_err(|e| e.to_string())
}

fn txt_to_md(input: &Path, output: &Path) -> Result<(), String> {
    fs::copy(input, output).map(|_| ()).map_err(|e| e.to_string())
}

fn txt_to_pdf(input: &Path, output: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let (doc, page, layer) = PdfDocument::new("RFileMaster Export", Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc.add_builtin_font(printpdf::BuiltinFont::Helvetica).map_err(|e| e.to_string())?;
    let layer = doc.get_page(page).get_layer(layer);
    layer.begin_text_section();
    layer.set_font(&font, 11.0);
    layer.set_text_cursor(Mm(15.0), Mm(280.0));
    layer.set_line_height(16.0);
    for line in text.lines() {
        layer.write_text(line, &font);
        layer.add_line_break();
    }
    layer.end_text_section();
    fs::write(output, doc.save_to_bytes().map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn txt_to_html(input: &Path, output: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let body: String = text.lines().map(|l| {
        if l.is_empty() { "<br>".to_string() } else { format!("<p>{}</p>", l) }
    }).collect::<Vec<_>>().join("\n");
    fs::write(output, format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><style>body{{font-family:sans-serif;max-width:800px;margin:40px auto;line-height:1.6;padding:0 20px}}</style></head><body>{}</body></html>",
        body
    )).map_err(|e| e.to_string())
}

fn rtf_to_txt(input: &Path, output: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let mut out = String::new();
    let mut in_group = 0i32;
    let mut skip = false;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '{' => { in_group += 1; }
            '}' => { in_group -= 1; skip = false; }
            '\\' => {
                let mut word = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '-' { word.push(c); chars.next(); } else { break; }
                }
                if chars.peek() == Some(&' ') { chars.next(); }
                skip = matches!(word.as_str(), "pntext"|"fonttbl"|"colortbl"|"stylesheet"|"info"|"*");
            }
            '\n'|'\r' => {}
            c if !skip && in_group == 0 => { out.push(c); }
            _ => {}
        }
    }
    let cleaned: String = out.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect::<Vec<_>>().join("\n");
    fs::write(output, cleaned).map_err(|e| e.to_string())
}

fn epub_to_txt(input: &Path, output: &Path) -> Result<(), String> {
    let mut doc = epub::doc::EpubDoc::new(input).map_err(|e| e.to_string())?;
    let mut all_text = String::new();
    let num = doc.get_num_pages();
    for i in 0..num {
        doc.set_current_page(i);
        if let Some((content, _)) = doc.get_current_str() {
            let plain = strip_html(&content);
            if !plain.trim().is_empty() {
                all_text.push_str(plain.trim());
                all_text.push_str("\n\n");
            }
        }
    }
    fs::write(output, all_text.trim()).map_err(|e| e.to_string())
}

fn epub_to_html(input: &Path, output: &Path) -> Result<(), String> {
    let mut doc = epub::doc::EpubDoc::new(input).map_err(|e| e.to_string())?;
    let mut body = String::new();
    let num = doc.get_num_pages();
    for i in 0..num {
        doc.set_current_page(i);
        if let Some((content, _)) = doc.get_current_str() {
            body.push_str(&content);
            body.push('\n');
        }
    }
    fs::write(output, format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><style>body{{font-family:serif;max-width:800px;margin:40px auto;line-height:1.8;padding:0 20px}}</style></head><body>{}</body></html>",
        body
    )).map_err(|e| e.to_string())
}

fn tar_to_tar_gz(input: &Path, output: &Path) -> Result<(), String> {
    let data = fs::read(input).map_err(|e| e.to_string())?;
    let out_file = fs::File::create(output).map_err(|e| e.to_string())?;
    let mut encoder = flate2::write::GzEncoder::new(out_file, flate2::Compression::default());
    encoder.write_all(&data).map_err(|e| e.to_string())?;
    encoder.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn sevenz_to_zip(input: &Path, output: &Path) -> Result<(), String> {
    let temp_dir = std::env::temp_dir().join(format!("rfilemaster_7z_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;
    sevenz_rust::decompress_file(input, &temp_dir).map_err(|e| e.to_string())?;
    let result = dir_to_zip(&temp_dir, output);
    let _ = fs::remove_dir_all(&temp_dir);
    result
}

fn sevenz_to_tar_gz(input: &Path, output: &Path) -> Result<(), String> {
    let temp_dir = std::env::temp_dir().join(format!("rfilemaster_7z_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;
    sevenz_rust::decompress_file(input, &temp_dir).map_err(|e| e.to_string())?;
    let result = dir_to_tar_gz(&temp_dir, output);
    let _ = fs::remove_dir_all(&temp_dir);
    result
}

fn dir_to_zip(dir: &Path, output: &Path) -> Result<(), String> {
    let out_file = fs::File::create(output).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(out_file);
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let rel = path.strip_prefix(dir).map_err(|e| e.to_string())?.to_string_lossy().to_string();
            if path.is_dir() {
                stack.push(path);
            } else {
                let contents = fs::read(&path).map_err(|e| e.to_string())?;
                zip.start_file(&rel, options).map_err(|e| e.to_string())?;
                zip.write_all(&contents).map_err(|e| e.to_string())?;
            }
        }
    }
    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn dir_to_tar_gz(dir: &Path, output: &Path) -> Result<(), String> {
    let out_file = fs::File::create(output).map_err(|e| e.to_string())?;
    let gz = flate2::write::GzEncoder::new(out_file, flate2::Compression::default());
    let mut tar = tar::Builder::new(gz);
    tar.append_dir_all(".", dir).map_err(|e| e.to_string())?;
    tar.finish().map_err(|e| e.to_string())
}

fn docx_extract_text(input: &Path) -> Result<String, String> {
    let file = fs::File::open(input).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut xml = String::new();
    zip.by_name("word/document.xml").map_err(|e| e.to_string())?.read_to_string(&mut xml).map_err(|e| e.to_string())?;
    let mut reader = quick_xml::Reader::from_str(&xml);
    reader.config_mut().trim_text(false);
    let mut out = String::new();
    let mut buf = Vec::new();
    let mut in_text = false;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(e)) => {
                let local = e.local_name();
                if local.as_ref() == b"t" { in_text = true; }
            }
            Ok(quick_xml::events::Event::End(e)) => {
                let local = e.local_name();
                if local.as_ref() == b"t" { in_text = false; }
                if local.as_ref() == b"p" { out.push('\n'); }
            }
            Ok(quick_xml::events::Event::Text(e)) => {
                if in_text {
                    out.push_str(&e.unescape().map_err(|e| e.to_string())?);
                }
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(e) => return Err(e.to_string()),
            _ => {}
        }
        buf.clear();
    }
    Ok(out.trim().to_string())
}

fn docx_to_txt(input: &Path, output: &Path) -> Result<(), String> {
    fs::write(output, docx_extract_text(input)?).map_err(|e| e.to_string())
}

fn docx_to_md(input: &Path, output: &Path) -> Result<(), String> {
    let text = docx_extract_text(input)?;
    let md: String = text.lines().map(|l| {
        if l.trim().is_empty() { String::new() } else { format!("{}\n", l) }
    }).collect();
    fs::write(output, md).map_err(|e| e.to_string())
}

fn txt_to_qr(input: &Path, output: &Path, target_ext: &str) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let code = qrcode::QrCode::new(text.trim().as_bytes()).map_err(|e| e.to_string())?;
    if target_ext == "svg" {
        let svg_str = code.render::<qrcode::render::svg::Color>().min_dimensions(400, 400).build();
        fs::write(output, svg_str).map_err(|e| e.to_string())
    } else {
        let img = code.render::<image::Luma<u8>>().min_dimensions(400, 400).build();
        image::DynamicImage::ImageLuma8(img).save_with_format(output, ImageFormat::Png).map_err(|e| e.to_string())
    }
}

fn csv_to_chart(input: &Path, output: &Path, target_ext: &str) -> Result<(), String> {
    let mut rdr = csv::Reader::from_path(input).map_err(|e| e.to_string())?;
    let headers = rdr.headers().map_err(|e| e.to_string())?.clone();
    if headers.len() < 2 {
        return Err("CSV needs at least two columns: a label column and a numeric column".to_string());
    }
    let mut labels: Vec<String> = Vec::new();
    let mut values: Vec<f64> = Vec::new();
    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        labels.push(record.get(0).unwrap_or("").to_string());
        values.push(record.get(1).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0));
    }
    if values.is_empty() {
        return Err("No data rows found in CSV".to_string());
    }
    let max_val = values.iter().cloned().fold(0.0_f64, f64::max) * 1.15;
    let title = format!("{} by {}", headers.get(1).unwrap_or("value"), headers.get(0).unwrap_or("label"));

    if target_ext == "svg" {
        render_bar_chart_svg(output, &labels, &values, max_val, &title)
    } else {
        render_bar_chart_png(output, &labels, &values, max_val, &title)
    }
}

fn render_bar_chart_png(output: &Path, labels: &[String], values: &[f64], max_val: f64, title: &str) -> Result<(), String> {
    use plotters::prelude::*;
    let root = BitMapBackend::new(output, (900, 540)).into_drawing_area();
    root.fill(&WHITE).map_err(|e| e.to_string())?;
    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 24))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0..labels.len(), 0.0..max_val)
        .map_err(|e| e.to_string())?;
    chart.configure_mesh()
        .x_labels(labels.len())
        .x_label_formatter(&|idx| labels.get(*idx).cloned().unwrap_or_default())
        .draw()
        .map_err(|e| e.to_string())?;
    chart.draw_series(values.iter().enumerate().map(|(i, v)| {
        let mut bar = Rectangle::new([(i, 0.0), (i + 1, *v)], BLUE.filled());
        bar.set_margin(0, 0, 5, 5);
        bar
    })).map_err(|e| e.to_string())?;
    root.present().map_err(|e| e.to_string())?;
    Ok(())
}

fn render_bar_chart_svg(output: &Path, labels: &[String], values: &[f64], max_val: f64, title: &str) -> Result<(), String> {
    use plotters::prelude::*;
    let root = SVGBackend::new(output, (900, 540)).into_drawing_area();
    root.fill(&WHITE).map_err(|e| e.to_string())?;
    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 24))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0..labels.len(), 0.0..max_val)
        .map_err(|e| e.to_string())?;
    chart.configure_mesh()
        .x_labels(labels.len())
        .x_label_formatter(&|idx| labels.get(*idx).cloned().unwrap_or_default())
        .draw()
        .map_err(|e| e.to_string())?;
    chart.draw_series(values.iter().enumerate().map(|(i, v)| {
        let mut bar = Rectangle::new([(i, 0.0), (i + 1, *v)], BLUE.filled());
        bar.set_margin(0, 0, 5, 5);
        bar
    })).map_err(|e| e.to_string())?;
    root.present().map_err(|e| e.to_string())?;
    Ok(())
}
fn zip_to_tar_gz(input: &Path, output: &Path) -> Result<(), String> {
    let mut zip = zip::ZipArchive::new(fs::File::open(input).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let gz = flate2::write::GzEncoder::new(fs::File::create(output).map_err(|e| e.to_string())?, flate2::Compression::default());
    let mut tar = tar::Builder::new(gz);
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
        if entry.is_dir() { continue; }
        let name = entry.name().to_string();
        let mut contents = Vec::new();
        entry.read_to_end(&mut contents).map_err(|e| e.to_string())?;
        let mut header = tar::Header::new_gnu();
        header.set_size(contents.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append_data(&mut header, &name, contents.as_slice()).map_err(|e| e.to_string())?;
    }
    tar.finish().map_err(|e| e.to_string())
}

fn tar_gz_to_zip(input: &Path, output: &Path) -> Result<(), String> {
    let data = fs::read(input).map_err(|e| e.to_string())?;
    let gz = flate2::read::GzDecoder::new(data.as_slice());
    let mut tar = tar::Archive::new(gz);
    let mut zip = zip::ZipWriter::new(fs::File::create(output).map_err(|e| e.to_string())?);
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for entry in tar.entries().map_err(|e| e.to_string())? {
        let mut entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path().map_err(|e| e.to_string())?.to_string_lossy().to_string();
        if entry.header().entry_type().is_file() {
            let mut contents = Vec::new();
            entry.read_to_end(&mut contents).map_err(|e| e.to_string())?;
            zip.start_file(&path, options).map_err(|e| e.to_string())?;
            zip.write_all(&contents).map_err(|e| e.to_string())?;
        }
    }
    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}