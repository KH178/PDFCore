use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    pub header: String,
    pub width: f64,
    #[serde(default = "default_text_align")]
    pub align: TextAlign,
    #[serde(default)]
    pub field: Option<String>, // For data binding
}

fn default_text_align() -> TextAlign {
    TextAlign::Left
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSettings {
    #[serde(default = "default_padding")]
    pub padding: f64,
    #[serde(default = "default_border")]
    pub border_width: f64,
    #[serde(default = "default_header_height")]
    pub header_height: f64,
    #[serde(default = "default_cell_height")]
    pub cell_height: f64,
    #[serde(default = "default_font_size")]
    pub font_size: f64,
    #[serde(default = "default_font_color")]
    pub font_color: crate::core::color::Color,
    #[serde(default = "default_header_bg")]
    pub header_bg: crate::core::color::Color,
    #[serde(default = "default_header_color")]
    pub header_color: crate::core::color::Color,
    #[serde(default = "default_border_color")]
    pub border_color: crate::core::color::Color,
    #[serde(default = "default_striped")]
    pub striped: bool,
    #[serde(default = "default_alternate_row_color")]
    pub alternate_row_color: crate::core::color::Color,
}

fn default_padding() -> f64 { 5.0 }
fn default_border() -> f64 { 1.0 }
fn default_header_height() -> f64 { 30.0 }
fn default_cell_height() -> f64 { 20.0 }
fn default_font_size() -> f64 { 10.0 }
fn default_font_color() -> crate::core::color::Color { crate::core::color::Color::black() }
fn default_header_bg() -> crate::core::color::Color { crate::core::color::Color::gray(0.9) }
fn default_header_color() -> crate::core::color::Color { crate::core::color::Color::black() }
fn default_border_color() -> crate::core::color::Color { crate::core::color::Color::black() }
fn default_striped() -> bool { false }
fn default_alternate_row_color() -> crate::core::color::Color { crate::core::color::Color::gray(0.95) }

impl Default for TableSettings {
    fn default() -> Self {
        TableSettings {
            padding: default_padding(),
            border_width: default_border(),
            header_height: default_header_height(),
            cell_height: default_cell_height(),
            font_size: default_font_size(),
            font_color: default_font_color(),
            header_bg: default_header_bg(),
            header_color: default_header_color(),
            border_color: default_border_color(),
            striped: default_striped(),
            alternate_row_color: default_alternate_row_color(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub content: String,
    #[serde(default = "default_span")]
    pub colspan: usize,
    #[serde(default = "default_span")]
    pub rowspan: usize,
}

fn default_span() -> usize { 1 }

#[derive(Debug, Clone)]
pub struct Table {
    pub columns: Vec<TableColumn>,
    pub rows: Vec<Vec<TableCell>>,
    pub settings: TableSettings,
}

impl Table {
    pub fn new(columns: Vec<TableColumn>) -> Self {
        Table {
            columns,
            rows: Vec::new(),
            settings: TableSettings::default(),
        }
    }

    pub fn add_row(&mut self, row: Vec<TableCell>) {
        let span_sum: usize = row.iter().map(|c| c.colspan).sum();
        if span_sum == self.columns.len() {
            self.rows.push(row);
        } else if span_sum < self.columns.len() {
            let mut r = row;
            let mut current_sum = span_sum;
            while current_sum < self.columns.len() {
                r.push(TableCell { content: String::new(), colspan: 1, rowspan: 1 });
                current_sum += 1;
            }
            self.rows.push(r);
        } else {
            self.rows.push(row);
        }
    }
}
