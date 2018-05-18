use cairo;
use pango;
use pangocairo;

use css::px2pt;

use std::cell::RefCell;
use pango::{ContextExt, LayoutExt};

use app_units::Au;

thread_local!(
    pub static PANGO_LAYOUT: RefCell<pango::Layout> = {
        let surface = cairo::ImageSurface::create(cairo::Format::Rgb24, 0, 0).unwrap();
        let ctx = pangocairo::functions::create_context(&cairo::Context::new(&surface)).unwrap();
        let layout = pango::Layout::new(&ctx);
        RefCell::new(layout)
    };
    pub static FONT_DESC: RefCell<pango::FontDescription> = {
        RefCell::new(pango::FontDescription::from_string("sans-serif normal 16"))
    }
);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Font {
    pub size: Au,
    pub weight: FontWeight,
    pub slant: FontSlant,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FontWeight {
    Normal,
    Bold,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FontSlant {
    Normal,
    Italic,
}

impl Font {
    pub fn new(size: Au, weight: FontWeight, slant: FontSlant) -> Font {
        Font {
            size: size,
            weight: weight,
            slant: slant,
        }
    }

    pub fn new_empty() -> Font {
        Font {
            size: Au(0),
            weight: FontWeight::Normal,
            slant: FontSlant::Normal,
        }
    }

    pub fn text_width(&self, text: &str) -> f64 {
        FONT_DESC.with(|font_desc| {
            let mut font_desc = font_desc.borrow_mut();
            font_desc.set_size(pango::units_from_double(px2pt(self.size.to_f64_px())));
            font_desc.set_style(self.slant.to_pango_font_slant());
            font_desc.set_weight(self.weight.to_pango_font_weight());
            PANGO_LAYOUT.with(|layout| {
                let layout = layout.borrow_mut();
                layout.set_text(text);
                layout.set_font_description(Some(&*font_desc));
                pango::units_to_double(layout.get_size().0)
            })
        })
    }

    pub fn get_ascent_descent(&self) -> (Au, Au) {
        FONT_DESC.with(|font_desc| {
            let mut font_desc = font_desc.borrow_mut();
            font_desc.set_size(pango::units_from_double(px2pt(self.size.to_f64_px())));
            font_desc.set_style(self.slant.to_pango_font_slant());
            font_desc.set_weight(self.weight.to_pango_font_weight());
            PANGO_LAYOUT.with(|layout| {
                let ctx = layout.borrow_mut().get_context().unwrap();
                let metrics =
                    ctx.get_metrics(Some(&*font_desc), Some(&pango::Language::from_string("")))
                        .unwrap();
                (
                    Au::from_f64_px(pango::units_to_double(metrics.get_ascent()) as f64),
                    Au::from_f64_px(pango::units_to_double(metrics.get_descent()) as f64),
                )
            })
        })
    }

    pub fn compute_max_chars(&self, s: &str, max_width: f64) -> usize {
        // TODO: Inefficient!
        if max_width < 0f64 {
            return 0;
        }

        let mut buf = "".to_string();
        let mut last_splittable_pos = None;
        let mut last_pos = 0;
        for (pos, c) in s.char_indices() {
            buf.push(c);

            if c.is_whitespace() || c.is_ascii_punctuation() {
                last_splittable_pos = Some(pos);
            }

            let text_width = self.text_width(buf.as_str());
            if text_width > max_width {
                if let Some(pos) = last_splittable_pos {
                    return pos + 1; // '1' means whitespace or punctuation.
                } else {
                    if pos == 0 {
                        break;
                    }
                    if pos - last_pos > 1 {
                        // if c is multi-byte character
                        return pos;
                    }
                }
            }

            last_pos = pos;
        }

        if s.is_empty() {
            0
        } else {
            1
        }
    }
}
