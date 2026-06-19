use std::path::{Path, PathBuf};
use std::fs;
use std::io::{Read, Write};
use image::ImageFormat;
use printpdf::{Mm, PdfDocument, Image as PdfImage, ImageTransform, ImageXObject, ColorSpace, ColorBits};

pub fn convert(inputs: &[PathBuf], source: &str, target: &str, output_path: &Path, merge: bool) -> Result<(), String> {
    if inputs.is_empty() {
        return Err("No input files provided".to_string());
    }
    match (source, target) {
        (s, "pdf") if is_image_format(s) => images_to_pdf(inputs, output_path),
        (s, t) if is_image_format(s) && is_image_format(t) => {
            batch_or_single(inputs, output_path, merge, |input, out| image_to_image(input, out, t))
        }
        ("json", "csv") => batch_or_single(inputs, output_path, merge, json_to_csv),
        ("csv", "json") => {
            if merge && inputs.len() > 1 {
                csvs_to_json_merged(inputs, output_path)
            } else {
                batch_or_single(inputs, output_path, merge, csv_to_json)
            }
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
        ("md", "html") => {
            if merge && inputs.len() > 1 {
                mds_to_html_merged(inputs, output_path)
            } else {
                batch_or_single(inputs, output_path, merge, md_to_html)
            }
        }
        ("html", "txt") => batch_or_single(inputs, output_path, merge, html_to_txt),
        ("txt", "md") => batch_or_single(inputs, output_path, merge, txt_to_md),
        ("txt", "html") => batch_or_single(inputs, output_path, merge, txt_to_html),
        ("epub", "txt") => batch_or_single(inputs, output_path, merge, epub_to_txt),
        ("epub", "html") => batch_or_single(inputs, output_path, merge, epub_to_html),
        ("zip", "tar_gz") => batch_or_single(inputs, output_path, merge, zip_to_tar_gz),
        ("tar_gz", "zip") | ("tar", "zip") => batch_or_single(inputs, output_path, merge, tar_gz_to_zip),
        (s, "zip") if matches!(s, "zip"|"tar"|"tar_gz"|"7z") => {
            batch_or_single(inputs, output_path, merge, repack_to_zip)
        }
        _ => Err(format!("Conversion from {} to {} requires an external tool and is not supported natively", source, target)),
    }
}

fn batch_or_single(
    inputs: &[PathBuf],
    output_path: &Path,
    _merge: bool,
    f: impl Fn(&Path, &Path) -> Result<(), String>,
) -> Result<(), String> {
    if inputs.len() == 1 {
        return f(&inputs[0], output_path);
    }
    let out_dir = output_path.parent().unwrap_or(Path::new("."));
    let ext = output_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    for input in inputs {
        let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
        let out = out_dir.join(format!("{}.{}", stem, ext));
        f(input, &out)?;
    }
    Ok(())
}

pub fn supports_merge(source: &str, target: &str) -> bool {
    (is_image_format(source) && target == "pdf")
        || (source == "md" && target == "html")
        || (source == "csv" && target == "json")
}

fn is_image_format(fmt: &str) -> bool {
    matches!(fmt, "png"|"jpg"|"jpeg"|"webp"|"bmp"|"tiff"|"gif"|"ico"|"avif")
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
    let fmt = ext_to_image_format(target_ext)
        .ok_or_else(|| format!("Unsupported image format: {}", target_ext))?;
    let img = image::open(input).map_err(|e| e.to_string())?;
    img.save_with_format(output, fmt).map_err(|e| e.to_string())
}

fn add_img_to_pdf_layer(img: image::DynamicImage, layer: printpdf::PdfLayerReference) {
    let (w, h) = (img.width(), img.height());
    let rgb = img.to_rgb8();
    let raw = rgb.into_raw();
    let xobj = ImageXObject {
        width: printpdf::Px(w as usize),
        height: printpdf::Px(h as usize),
        color_space: ColorSpace::Rgb,
        bits_per_component: ColorBits::Bit8,
        interpolate: true,
        image_data: raw,
        image_filter: None,
        smask: None,
        clipping_bbox: None,
    };
    PdfImage::from(xobj).add_to_layer(layer, ImageTransform {
        translate_x: Some(Mm(0.0)),
        translate_y: Some(Mm(0.0)),
        scale_x: None,
        scale_y: None,
        dpi: Some(150.0),
        ..Default::default()
    });
}

fn images_to_pdf(inputs: &[PathBuf], output: &Path) -> Result<(), String> {
    let dpi = 150.0_f32;
    let mm_per_px = 25.4 / dpi;
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
    let bytes = doc.save_to_bytes().map_err(|e| e.to_string())?;
    fs::write(output, bytes).map_err(|e| e.to_string())
}

fn json_to_csv(input: &Path, output: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let rows = value.as_array().ok_or("JSON root must be an array of objects")?;
    if rows.is_empty() {
        return fs::write(output, "").map_err(|e| e.to_string());
    }
    let headers: Vec<String> = rows[0].as_object()
        .ok_or("Each array element must be a JSON object")?
        .keys().cloned().collect();
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
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let value: toml::Value = serde_json::from_str::<serde_json::Value>(&text)
        .map_err(|e| e.to_string())
        .and_then(|v| toml::Value::try_from(v).map_err(|e| e.to_string()))?;
    fs::write(output, toml::to_string_pretty(&value).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn toml_to_json(input: &Path, output: &Path) -> Result<(), String> {
    let value: toml::Value = toml::from_str(&fs::read_to_string(input).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    fs::write(output, serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn yaml_to_toml(input: &Path, output: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let value: serde_yaml::Value = serde_yaml::from_str(&text).map_err(|e| e.to_string())?;
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

fn xml_to_json(input: &Path, output: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let value = xml_text_to_value(&text)?;
    fs::write(output, serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn json_to_xml(input: &Path, output: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let xml = value_to_xml(&value, "root");
    fs::write(output, format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", xml)).map_err(|e| e.to_string())
}

fn xml_to_csv(input: &Path, output: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let value = xml_text_to_value(&text)?;
    let tmp = std::env::temp_dir().join("rfilemaster_xml_tmp.json");
    fs::write(&tmp, serde_json::to_string(&value).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    json_to_csv(&tmp, output)?;
    let _ = fs::remove_file(tmp);
    Ok(())
}

fn xml_to_yaml(input: &Path, output: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let value = xml_text_to_value(&text)?;
    fs::write(output, serde_yaml::to_string(&value).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn yaml_to_xml(input: &Path, output: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let value: serde_yaml::Value = serde_yaml::from_str(&text).map_err(|e| e.to_string())?;
    let json_val: serde_json::Value = serde_json::to_value(&value).map_err(|e| e.to_string())?;
    let xml = value_to_xml(&json_val, "root");
    fs::write(output, format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", xml)).map_err(|e| e.to_string())
}

fn xml_text_to_value(text: &str) -> Result<serde_json::Value, String> {
    let mut reader = quick_xml::Reader::from_str(text);
    reader.config_mut().trim_text(true);
    let mut stack: Vec<(String, Vec<serde_json::Value>)> = Vec::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                stack.push((name, Vec::new()));
            }
            Ok(quick_xml::events::Event::End(_)) => {
                if let Some((name, children)) = stack.pop() {
                    let val = if children.is_empty() {
                        serde_json::Value::Null
                    } else if children.len() == 1 {
                        children.into_iter().next().unwrap()
                    } else {
                        serde_json::Value::Array(children)
                    };
                    if let Some(parent) = stack.last_mut() {
                        let entry = serde_json::json!({ &name: val });
                        parent.1.push(entry);
                    } else {
                        return Ok(serde_json::json!({ &name: val }));
                    }
                }
            }
            Ok(quick_xml::events::Event::Text(e)) => {
                let text = e.unescape().map_err(|e| e.to_string())?;
                if !text.trim().is_empty() {
                    if let Some(parent) = stack.last_mut() {
                        parent.1.push(serde_json::Value::String(text.to_string()));
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
        serde_json::Value::Array(arr) => {
            arr.iter().map(|v| value_to_xml(v, tag)).collect()
        }
        serde_json::Value::Null => format!("<{}/>", tag),
        other => format!("<{}>{}</{}>", tag, other.to_string().trim_matches('"'), tag),
    }
}

fn md_to_html(input: &Path, output: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let parser = pulldown_cmark::Parser::new(&text);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    let full = format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><style>body{{font-family:sans-serif;max-width:800px;margin:40px auto;line-height:1.6;padding:0 20px}}</style></head><body>{}</body></html>",
        html
    );
    fs::write(output, full).map_err(|e| e.to_string())
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
    let full = format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><style>body{{font-family:sans-serif;max-width:800px;margin:40px auto;line-height:1.6;padding:0 20px}}</style></head><body>{}</body></html>",
        html
    );
    fs::write(output, full).map_err(|e| e.to_string())
}

fn html_to_txt(input: &Path, output: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let mut out = String::new();
    let mut in_tag = false;
    for ch in text.chars() {
        match ch {
            '<' => { in_tag = true; }
            '>' => { in_tag = false; out.push(' '); }
            c if !in_tag => { out.push(c); }
            _ => {}
        }
    }
    let cleaned: String = out.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect::<Vec<_>>().join("\n");
    fs::write(output, cleaned).map_err(|e| e.to_string())
}

fn txt_to_md(input: &Path, output: &Path) -> Result<(), String> {
    fs::copy(input, output).map(|_| ()).map_err(|e| e.to_string())
}

fn txt_to_html(input: &Path, output: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let body: String = text.lines().map(|l| {
        if l.is_empty() { "<br>".to_string() } else { format!("<p>{}</p>", l) }
    }).collect::<Vec<_>>().join("\n");
    let full = format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><style>body{{font-family:sans-serif;max-width:800px;margin:40px auto;line-height:1.6;padding:0 20px}}</style></head><body>{}</body></html>",
        body
    );
    fs::write(output, full).map_err(|e| e.to_string())
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
    let full = format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><style>body{{font-family:serif;max-width:800px;margin:40px auto;line-height:1.8;padding:0 20px}}</style></head><body>{}</body></html>",
        body
    );
    fs::write(output, full).map_err(|e| e.to_string())
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

fn zip_to_tar_gz(input: &Path, output: &Path) -> Result<(), String> {
    let zip_file = fs::File::open(input).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(zip_file).map_err(|e| e.to_string())?;
    let out_file = fs::File::create(output).map_err(|e| e.to_string())?;
    let gz = flate2::write::GzEncoder::new(out_file, flate2::Compression::default());
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
    let out_file = fs::File::create(output).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(out_file);
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

fn repack_to_zip(input: &Path, output: &Path) -> Result<(), String> {
    let zip_file = fs::File::open(input).map_err(|e| e.to_string())?;
    let mut zip_in = zip::ZipArchive::new(zip_file).map_err(|e| e.to_string())?;
    let out_file = fs::File::create(output).map_err(|e| e.to_string())?;
    let mut zip_out = zip::ZipWriter::new(out_file);
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for i in 0..zip_in.len() {
        let mut entry = zip_in.by_index(i).map_err(|e| e.to_string())?;
        if entry.is_dir() { continue; }
        let name = entry.name().to_string();
        let mut contents = Vec::new();
        entry.read_to_end(&mut contents).map_err(|e| e.to_string())?;
        zip_out.start_file(&name, options).map_err(|e| e.to_string())?;
        zip_out.write_all(&contents).map_err(|e| e.to_string())?;
    }
    zip_out.finish().map_err(|e| e.to_string())?;
    Ok(())
}