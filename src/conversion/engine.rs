use std::path::{Path, PathBuf};
use std::fs;
use image::ImageFormat;
use printpdf::{Mm, PdfDocument, Image as PdfImage, ImageTransform, ImageXObject, ColorSpace, ColorBits};

pub fn convert(inputs: &[PathBuf], source: &str, target: &str, output_path: &Path) -> Result<(), String> {
    match (source, target) {
        (s, "pdf") if is_image_format(s) => images_to_pdf(inputs, output_path),
        (s, t) if is_image_format(s) && is_image_format(t) => image_to_image(&inputs[0], output_path, t),
        ("json", "csv") => json_to_csv(&inputs[0], output_path),
        ("csv", "json") => csv_to_json(&inputs[0], output_path),
        ("json", "yaml") => json_to_yaml(&inputs[0], output_path),
        ("yaml", "json") => yaml_to_json(&inputs[0], output_path),
        ("json", "toml") => json_to_toml(&inputs[0], output_path),
        ("toml", "json") => toml_to_json(&inputs[0], output_path),
        ("yaml", "toml") => yaml_to_toml(&inputs[0], output_path),
        ("toml", "yaml") => toml_to_yaml(&inputs[0], output_path),
        ("csv", "tsv") => csv_to_tsv(&inputs[0], output_path),
        ("tsv", "csv") => tsv_to_csv(&inputs[0], output_path),
        ("md", "html") => md_to_html(&inputs[0], output_path),
        ("html", "txt") => html_to_txt(&inputs[0], output_path),
        ("txt", "md") => txt_to_md(&inputs[0], output_path),
        _ => Err(format!("Conversion from {} to {} is not yet implemented", source, target)),
    }
}

fn is_image_format(fmt: &str) -> bool {
    matches!(fmt, "png" | "jpg" | "jpeg" | "webp" | "bmp" | "tiff" | "gif" | "ico" | "avif")
}

fn ext_to_image_format(ext: &str) -> Option<ImageFormat> {
    match ext {
        "png" => Some(ImageFormat::Png),
        "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
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

fn images_to_pdf(inputs: &[PathBuf], output: &Path) -> Result<(), String> {
    if inputs.is_empty() {
        return Err("No input files provided".to_string());
    }

    let first = image::open(&inputs[0]).map_err(|e| e.to_string())?;
    let (fw, fh) = (first.width() as f32, first.height() as f32);
    let dpi = 150.0_f32;
    let page_w = Mm((fw / dpi) * 25.4);
    let page_h = Mm((fh / dpi) * 25.4);

    let (doc, first_page, first_layer) = PdfDocument::new("RFileMaster Export", page_w, page_h, "Layer 1");

    let add_image_to_layer = |img: image::DynamicImage, layer: printpdf::PdfLayerReference| -> Result<(), String> {
        let (w, h) = (img.width(), img.height());
        let rgb = img.to_rgb8();
        let raw = rgb.into_raw();
        let image_xobj = ImageXObject {
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
        let pdf_img = PdfImage::from(image_xobj);
        let img_w_mm = (w as f32 / dpi) * 25.4;
        let img_h_mm = (h as f32 / dpi) * 25.4;
        pdf_img.add_to_layer(
            layer,
            ImageTransform {
                translate_x: Some(Mm(0.0)),
                translate_y: Some(Mm(0.0)),
                scale_x: Some(img_w_mm / (w as f32)),
                scale_y: Some(img_h_mm / (h as f32)),
                ..Default::default()
            },
        );
        Ok(())
    };

    let first_layer_ref = doc.get_page(first_page).get_layer(first_layer);
    add_image_to_layer(first, first_layer_ref)?;

    for input in inputs.iter().skip(1) {
        let img = image::open(input).map_err(|e| e.to_string())?;
        let (iw, ih) = (img.width() as f32, img.height() as f32);
        let pw = Mm((iw / dpi) * 25.4);
        let ph = Mm((ih / dpi) * 25.4);
        let (page_idx, layer_idx) = doc.add_page(pw, ph, "Layer 1");
        let layer_ref = doc.get_page(page_idx).get_layer(layer_idx);
        add_image_to_layer(img, layer_ref)?;
    }

    let bytes = doc.save_to_bytes().map_err(|e| e.to_string())?;
    fs::write(output, bytes).map_err(|e| e.to_string())
}

fn json_to_csv(input: &Path, output: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let rows = value.as_array().ok_or("JSON root must be an array of objects")?;
    if rows.is_empty() {
        fs::write(output, "").map_err(|e| e.to_string())?;
        return Ok(());
    }
    let headers: Vec<String> = rows[0]
        .as_object()
        .ok_or("Each array element must be a JSON object")?
        .keys()
        .cloned()
        .collect();
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
    let json = serde_json::to_string_pretty(&records).map_err(|e| e.to_string())?;
    fs::write(output, json).map_err(|e| e.to_string())
}

fn json_to_yaml(input: &Path, output: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let yaml = serde_yaml::to_string(&value).map_err(|e| e.to_string())?;
    fs::write(output, yaml).map_err(|e| e.to_string())
}

fn yaml_to_json(input: &Path, output: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let value: serde_yaml::Value = serde_yaml::from_str(&text).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
    fs::write(output, json).map_err(|e| e.to_string())
}

fn json_to_toml(input: &Path, output: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let value: toml::Value = serde_json::from_str::<serde_json::Value>(&text)
        .map_err(|e| e.to_string())
        .and_then(|v| toml::Value::try_from(v).map_err(|e| e.to_string()))?;
    let out = toml::to_string_pretty(&value).map_err(|e| e.to_string())?;
    fs::write(output, out).map_err(|e| e.to_string())
}

fn toml_to_json(input: &Path, output: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let value: toml::Value = toml::from_str(&text).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
    fs::write(output, json).map_err(|e| e.to_string())
}

fn yaml_to_toml(input: &Path, output: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let value: serde_yaml::Value = serde_yaml::from_str(&text).map_err(|e| e.to_string())?;
    let json_val: serde_json::Value = serde_json::to_value(&value).map_err(|e| e.to_string())?;
    let toml_val: toml::Value = toml::Value::try_from(json_val).map_err(|e| e.to_string())?;
    let out = toml::to_string_pretty(&toml_val).map_err(|e| e.to_string())?;
    fs::write(output, out).map_err(|e| e.to_string())
}

fn toml_to_yaml(input: &Path, output: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let value: toml::Value = toml::from_str(&text).map_err(|e| e.to_string())?;
    let yaml = serde_yaml::to_string(&value).map_err(|e| e.to_string())?;
    fs::write(output, yaml).map_err(|e| e.to_string())
}

fn csv_to_tsv(input: &Path, output: &Path) -> Result<(), String> {
    let mut rdr = csv::Reader::from_path(input).map_err(|e| e.to_string())?;
    let mut wtr = csv::WriterBuilder::new().delimiter(b'\t').from_path(output).map_err(|e| e.to_string())?;
    let headers = rdr.headers().map_err(|e| e.to_string())?.clone();
    wtr.write_record(&headers).map_err(|e| e.to_string())?;
    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        wtr.write_record(&record).map_err(|e| e.to_string())?;
    }
    wtr.flush().map_err(|e| e.to_string())
}

fn tsv_to_csv(input: &Path, output: &Path) -> Result<(), String> {
    let mut rdr = csv::ReaderBuilder::new().delimiter(b'\t').from_path(input).map_err(|e| e.to_string())?;
    let mut wtr = csv::Writer::from_path(output).map_err(|e| e.to_string())?;
    let headers = rdr.headers().map_err(|e| e.to_string())?.clone();
    wtr.write_record(&headers).map_err(|e| e.to_string())?;
    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        wtr.write_record(&record).map_err(|e| e.to_string())?;
    }
    wtr.flush().map_err(|e| e.to_string())
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
    let cleaned: String = out.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(output, cleaned).map_err(|e| e.to_string())
}

fn txt_to_md(input: &Path, output: &Path) -> Result<(), String> {
    fs::copy(input, output).map(|_| ()).map_err(|e| e.to_string())
}