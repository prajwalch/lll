pub const PAGE_TEMPLATE: &str = include_str!("static/template.html");
pub const FILE_SVG_ICON: &str = include_str!("static/file.svg");
pub const FOLDER_SVG_ICON: &str = include_str!("static/folder.svg");

#[rustfmt::skip]
pub fn build_not_found_page() -> String {
    PAGE_TEMPLATE
        .replace("{title}", "Error Response")
        .replace("{content}", "<h1>404 Not Found</h1><p>Nothing matches the given URI</p>")
}
