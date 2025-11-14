use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct FileViewer {
    file_type: String,
    content: Vec<u8>,
}

#[wasm_bindgen]
impl FileViewer {
    #[wasm_bindgen(constructor)]
    pub fn new(file_type: String) -> FileViewer {
        console::log_1(&"FileViewer initialized".into());
        FileViewer {
            file_type,
            content: Vec::new(),
        }
    }

    #[wasm_bindgen]
    pub fn load_content(&mut self, data: Vec<u8>) {
        self.content = data;
        console::log_1(&format!("Loaded {} bytes", self.content.len()).into());
    }

    #[wasm_bindgen]
    pub fn render(&self) -> String {
        match self.file_type.as_str() {
            "text/plain" | "text/markdown" => self.render_text(),
            "image/png" | "image/jpeg" | "image/jpg" | "image/gif" | "image/webp" => {
                self.render_image()
            }
            "application/pdf" => self.render_pdf_placeholder(),
            "text/csv" => self.render_csv(),
            _ => self.render_unsupported(),
        }
    }

    fn render_text(&self) -> String {
        match String::from_utf8(self.content.clone()) {
            Ok(text) => format!(
                r#"<div class="text-viewer"><pre>{}</pre></div>"#,
                html_escape(&text)
            ),
            Err(_) => r#"<div class="error">Unable to decode text file</div>"#.to_string(),
        }
    }

    fn render_image(&self) -> String {
        let base64_data = base64::encode(&self.content);
        format!(
            r#"<div class="image-viewer"><img src="data:{};base64,{}" alt="File preview" /></div>"#,
            self.file_type, base64_data
        )
    }

    fn render_pdf_placeholder(&self) -> String {
        r#"<div class="pdf-viewer">
            <p>PDF Preview</p>
            <p>Use browser's PDF viewer or download to view full document</p>
        </div>"#
            .to_string()
    }

    fn render_csv(&self) -> String {
        match String::from_utf8(self.content.clone()) {
            Ok(text) => {
                let lines: Vec<&str> = text.lines().take(10).collect();
                let mut html = String::from(r#"<div class="csv-viewer"><table>"#);

                for (i, line) in lines.iter().enumerate() {
                    let cells: Vec<&str> = line.split(',').collect();
                    let tag = if i == 0 { "th" } else { "td" };

                    html.push_str("<tr>");
                    for cell in cells {
                        html.push_str(&format!("<{}>{}</{}>", tag, html_escape(cell), tag));
                    }
                    html.push_str("</tr>");
                }

                html.push_str("</table></div>");
                html
            }
            Err(_) => r#"<div class="error">Unable to decode CSV file</div>"#.to_string(),
        }
    }

    fn render_unsupported(&self) -> String {
        format!(
            r#"<div class="unsupported-viewer">
                <p>Preview not available for file type: {}</p>
                <p>Download file to view</p>
            </div>"#,
            self.file_type
        )
    }

    #[wasm_bindgen]
    pub fn get_file_info(&self) -> String {
        format!(
            r#"{{"type": "{}", "size": {}}}"#,
            self.file_type,
            self.content.len()
        )
    }
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello from WASM, {}!", name)
}

#[wasm_bindgen]
pub fn detect_file_type(filename: &str) -> String {
    let extension = filename.split('.').last().unwrap_or("");

    match extension.to_lowercase().as_str() {
        "txt" => "text/plain",
        "md" => "text/markdown",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "pdf" => "application/pdf",
        "csv" => "text/csv",
        "json" => "application/json",
        "xml" => "application/xml",
        _ => "application/octet-stream",
    }
    .to_string()
}
