use util::{Rgba, V2, Rect, Color, color, Anchor};
use util::text;
use canvas::{Canvas, FONT_H};
use canvas_util::{CanvasUtil};

pub enum Align {
    Left,
    Right,
    Center
}

pub struct Fonter<'a> {
    canvas: &'a mut Canvas,
    anchor: Anchor,
    align: Align,
    color: Rgba,
    z: f32,
    max_lines: Option<usize>,
    border: Option<Rgba>,
    max_width: Option<f32>,
    /// (Text, width) pairs.
    lines: Vec<(String, f32)>,
    longest_line_width: f32,
}

impl<'a> Fonter<'a> {
    pub fn new(canvas: &'a mut Canvas) -> Fonter<'a> {
        Fonter {
            canvas: canvas,
            anchor: Anchor::TopLeft,
            align: Align::Left,
            color: Color::from_color(&color::WHITE),
            z: 0.1,
            max_lines: None,
            border: None,
            max_width: None,
            lines: vec![(String::new(), 0.0)],
            longest_line_width: 0.0,
        }
    }

    /// Set the point of the text box which draw offset will anchor to.
    pub fn anchor(mut self, anchor: Anchor) -> Fonter<'a> {
        self.anchor = anchor; self
    }

    /// Set the text alignment
    pub fn align(mut self, align: Align) -> Fonter<'a> {
        self.align = align; self
    }

    /// Set text color. The default color is white.
    pub fn color<C: Color>(mut self, color: &C) -> Fonter<'a> {
        self.color = Color::from_color(color); self
    }

    /// Set border color. Before this is set, the drawn text will not be drawn
    /// with a border.
    pub fn border<C: Color>(mut self, color: &C) -> Fonter<'a> {
        self.border = Some(Color::from_color(color)); self
    }

    /// Set the z-layer to draw in.
    pub fn layer(mut self, z: f32) -> Fonter<'a> {
        self.z = z; self
    }

    /// Set the maximum width of the text area in pixels
    pub fn width(mut self, w: f32) -> Fonter<'a> {
        self.max_width = Some(w); self
    }

    /// Set the maximum number of lines to draw (lines of text before this are
    /// dropped).
    pub fn max_lines(mut self, max_lines: usize) -> Fonter<'a> {
        self.max_lines = Some(max_lines);
        self.cull_lines();
        self
    }

    /// Append to the fonter text.
    pub fn text(mut self, txt: String) -> Fonter<'a> {
        assert!(self.lines.len() > 0);
        // The last line can be added to, snip it off.
        let mut new_txt = format!("{}{}", self.lines[self.lines.len() - 1].0, txt);
        let new_len = self.lines.len() - 1;
        self.lines.truncate(new_len);
        if let Some(w) = self.max_width {
            new_txt = text::wrap_lines(&new_txt[..], &|c| self.canvas.char_width(c), w);
        }

        let mut new_lines: Vec<(String, f32)> = new_txt.split('\n').map(|s| (s.to_string(), self.str_width(s))).collect();
        self.lines.append(&mut new_lines);
        assert!(self.lines.len() > 0);

        self.cull_lines();
        self
    }

    fn cull_lines(&mut self) {
        if let Some(n) = self.max_lines {
            if self.lines.len() > n {
                let new_len = self.lines.len() - n;
                self.lines = self.lines.split_off(new_len);
            }
        }
        self.set_longest_width();
    }

    fn set_longest_width(&mut self) {
        self.longest_line_width = self.lines.iter().map(|x| x.1)
            .fold(0.0, |a, w| if w > a { w } else { a });
    }

    pub fn draw(&mut self, offset: V2<f32>) {
        let anchor_points = Rect(
            V2(0.0, 0.0),
            V2(self.longest_line_width,
               (self.lines.len() * FONT_H as usize) as f32));
        let offset = offset - anchor_points.point(self.anchor);
        // TODO Anchoring
        for (row, s) in self.lines.iter().enumerate() {
            let y = offset.1 + FONT_H as f32 + row as f32 * FONT_H as f32;
            let line_width = s.1;
            let mut x = offset.0 + match self.align {
                Align::Left => 0.0,
                Align::Right => (self.longest_line_width - line_width),
                Align::Center => (self.longest_line_width - line_width) / 2.0,
            };
            for c in s.0.chars() {
                self.canvas.draw_char(c, V2(x, y), self.z, &self.color, self.border.as_ref());
                x += self.canvas.char_width(c);
            }
        }
    }


    fn str_width(&self, s: &str) -> f32 {
        s.chars().fold(0.0, |a, c| a + self.canvas.char_width(c))
    }

}
