use anyhow::{Error, Result};
use font_kit::{
    family_name::FamilyName,
    handle::Handle,
    properties::{Properties, Style},
    source::{SystemSource},
};
use genpdf::{
    elements::{FrameCellDecorator, Paragraph, TableLayout, Text},
    fonts::{FontData, FontFamily},
    Mm, Document, SimplePageDecorator, Margins, Size,
};
use serde::{Deserialize, Serialize};

fn in_to_mm(inches: f32) -> Mm {
    Mm::from(inches * 25.4)
}

fn generate_pdf(font_family: FontFamily<FontData>, layout: PageLayout) {
    // Create a document and set the default font family
    let mut doc = Document::new(font_family);
    // Change the default settings
    doc.set_title("Generated Document");
    doc.set_paper_size(Size::new(
        in_to_mm(layout.width),
        in_to_mm(layout.height),
    ));
    let mut decorator = SimplePageDecorator::new();

    decorator.set_margins(Margins::trbl(
        in_to_mm(layout.margin.top),
        in_to_mm(layout.margin.right),
        in_to_mm(layout.margin.bottom),
        in_to_mm(layout.margin.left),
    ));

    doc.set_page_decorator(decorator);

    let _styled_text = genpdf::elements::StyledElement::new(
        Text::new("Hello World!"),
        genpdf::style::Style::new().with_font_size(25),
    );

    /*

    Welp, this is where this project ends for now.

    At the time of this writing it seems that genpdf doesn't support horizontal positioning very well.

    The original goal was to be able to create a configurable grid to print things on label paper (like Avery labels).

    Since it seems shapes are a bit tricky to to position, I'm leaving this code as-is. I'll probably re-write this in Go (assuming the PDF libraries are in a more completed state)

    */

    let mut table = TableLayout::new(vec![1, 1]);
    table.set_cell_decorator(FrameCellDecorator::new(true, true, false));
    let mut row = table.row();
    row.push_element(Paragraph::new("Cell 1"));
    row.push_element(Paragraph::new("Cell 2"));
    row.push().expect("Invalid table row");

    doc.push(Paragraph::new("This is a demo document."));

    doc.push(table);
    // Render the document and write it to a file
    doc.render_to_file("output.pdf")
        .expect("Failed to write PDF file");
}

#[derive(Serialize, Deserialize, Debug)]
struct BoundingBox {
    width: f64,
    height: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Quad<T: Copy> {
    top: T,
    right: T,
    bottom: T,
    left: T,
}

#[derive(Serialize, Deserialize, Debug)]
struct PageLayout {
    width: f32,
    height: f32,

    margin: Quad<f32>,

    label_size: BoundingBox,

    row_spacing: f32,
    column_spacing: f32,
}

// Based on an Avery 18160 label
// https://www.avery.com/templates/18160
const PAGE_LAYOUT: PageLayout = PageLayout {
    width: 8.5,
    height: 11.0,
    margin: Quad {
        top: 0.5,     // 1/2 inch
        right: 0.125, // 1/8 inch
        bottom: 0.5,
        left: 0.125,
    },
    label_size: BoundingBox {
        width: 2.0 + (5.0 / 8.0), // 2 & 5/8 inch
        height: 1.0,              // 1 inch
    },

    row_spacing: 0.0,
    column_spacing: 0.25,
};

fn font_handle_to_font_data(font_handle: &Handle) -> FontData {
    match font_handle {
        Handle::Path {
            path,
            font_index: _,
        } => return FontData::load(&path, None).unwrap(),
        Handle::Memory {
            bytes,
            font_index: _,
        } => return FontData::new(bytes.to_vec(), None).unwrap(),
    }
}

fn load_fonts(font_family_name: &str) -> Result<FontFamily<FontData>> {
    let mut genpdf_font_family = FontFamily {
        regular: None,
        bold: None,
        italic: None,
        bold_italic: None,
    };

    [
        Properties::new().style(Style::Normal),
        Properties::new().style(Style::Oblique),
        Properties::new().style(Style::Italic),
    ]
    .iter()
    .for_each(|prop| {
        let system_font = SystemSource::new()
            .select_best_match(&[FamilyName::Title(font_family_name.to_string())], prop);
        match system_font {
            Ok(font) => match prop.style {
                Style::Normal => genpdf_font_family.regular = Some(font_handle_to_font_data(&font)),
                Style::Oblique => genpdf_font_family.bold = Some(font_handle_to_font_data(&font)),
                Style::Italic => genpdf_font_family.italic = Some(font_handle_to_font_data(&font)),
            },
            Err(_) => {
                println!("Failed to load font");
                return;
            }
        };
    });

    let regular_font = genpdf_font_family
        .regular
        .ok_or(Error::msg("No regular font available"))?;

    return Ok(FontFamily {
        regular: regular_font.clone(),
        bold: match genpdf_font_family.bold {
            Some(font) => font,
            None => regular_font.clone(),
        },
        italic: match genpdf_font_family.italic {
            Some(font) => font,
            None => regular_font.clone(),
        },
        bold_italic: regular_font,
    });
}

fn main() {
    let font_family_name = "Arial";

    let font_family = load_fonts(font_family_name).expect("Failed to load font family");

    generate_pdf(font_family, PAGE_LAYOUT);
}
