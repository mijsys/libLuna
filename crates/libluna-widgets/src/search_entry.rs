//! `SearchEntry` — pole tekstowe wyszukiwania (lupka, tekst, kursor, „×” czyszczenia).
//!
//! Widget immediate: trzyma treść i pozycję kursora. Aplikacja dostarcza zdarzenia klawiatury
//! jako wywołania metod ([`SearchEntry::insert_str`], [`SearchEntry::backspace`],
//! [`SearchEntry::move_left`]…), a sama decyduje o fokusie ([`SearchEntry::set_focused`]).

use libluna_core::style::weight;
use libluna_core::{Point, Rect, Theme};

use crate::ctx::PaintCtx;
use crate::icon;

/// Pole wyszukiwania.
#[derive(Debug, Clone, Default)]
pub struct SearchEntry {
    text: String,
    /// Pozycja kursora jako indeks bajtowy (zawsze na granicy znaku).
    cursor: usize,
    placeholder: String,
    focused: bool,
    hover_clear: bool,
}

impl SearchEntry {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            ..Self::default()
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.cursor = self.text.len();
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = placeholder.into();
    }

    pub fn set_hover_clear(&mut self, hovered: bool) {
        self.hover_clear = hovered;
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }

    /// Wstawia tekst w miejscu kursora.
    pub fn insert_str(&mut self, s: &str) {
        self.text.insert_str(self.cursor, s);
        self.cursor += s.len();
    }

    pub fn insert_char(&mut self, c: char) {
        let mut buf = [0u8; 4];
        self.insert_str(c.encode_utf8(&mut buf));
    }

    /// Usuwa znak przed kursorem.
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            let prev = prev_boundary(&self.text, self.cursor);
            self.text.replace_range(prev..self.cursor, "");
            self.cursor = prev;
        }
    }

    /// Usuwa znak za kursorem.
    pub fn delete_forward(&mut self) {
        if self.cursor < self.text.len() {
            let next = next_boundary(&self.text, self.cursor);
            self.text.replace_range(self.cursor..next, "");
        }
    }

    pub fn move_left(&mut self) {
        self.cursor = prev_boundary(&self.text, self.cursor);
    }

    pub fn move_right(&mut self) {
        self.cursor = next_boundary(&self.text, self.cursor);
    }

    pub fn home(&mut self) {
        self.cursor = 0;
    }

    pub fn end(&mut self) {
        self.cursor = self.text.len();
    }

    pub fn height(&self, theme: &Theme) -> f64 {
        theme.metrics.entry.height
    }

    pub fn contains(&self, rect: Rect, p: Point) -> bool {
        rect.contains(p)
    }

    /// Prostokąt przycisku czyszczenia („×”).
    pub fn clear_rect(&self, rect: Rect, theme: &Theme) -> Rect {
        let m = theme.metrics.entry;
        Rect::from_xywh(
            rect.right() - m.pad_x - m.icon,
            rect.center().y - m.icon / 2.0,
            m.icon,
            m.icon,
        )
    }

    /// Czy punkt trafia w przycisk czyszczenia (tylko gdy pole niepuste).
    pub fn clear_at(&self, rect: Rect, theme: &Theme, p: Point) -> bool {
        !self.text.is_empty() && self.clear_rect(rect, theme).contains(p)
    }

    pub fn paint(&self, ctx: &mut PaintCtx, rect: Rect) {
        let theme = *ctx.theme;
        let m = theme.metrics.entry;
        let colors = theme.colors;
        let radius = theme.radius.medium;

        ctx.scene.fill_rounded_rect(rect, radius, colors.shade);
        let (border, width) = if self.focused {
            (colors.accent, 2.0)
        } else {
            (colors.border, 1.0)
        };
        ctx.scene.stroke_rounded_rect(rect, radius, width, border);

        let cy = rect.center().y;
        icon::search(
            ctx,
            colors.dim_text,
            Point::new(rect.x() + m.pad_x + m.icon / 2.0, cy),
            m.icon,
        );
        let text_x = rect.x() + m.pad_x + m.icon + 8.0;

        if self.text.is_empty() {
            ctx.label_left(
                text_x,
                cy,
                &self.placeholder,
                theme.typography.body.size,
                weight::NORMAL,
                colors.dim_text,
            );
        } else {
            let clear_space = if self.text.is_empty() {
                0.0
            } else {
                m.icon + 8.0
            };
            let text_right = rect.right() - m.pad_x - clear_space;
            ctx.scene.push_clip(Rect::from_xywh(
                text_x,
                rect.y(),
                (text_right - text_x).max(0.0),
                rect.height(),
            ));
            ctx.label_left(
                text_x,
                cy,
                &self.text,
                theme.typography.body.size,
                weight::NORMAL,
                colors.text,
            );
            if self.focused {
                let cx = text_x
                    + ctx.measure(
                        &self.text[..self.cursor],
                        theme.typography.body.size,
                        weight::NORMAL,
                    );
                ctx.scene.fill_rect(
                    Rect::from_xywh(cx, rect.y() + 6.0, 1.5, (rect.height() - 12.0).max(0.0)),
                    colors.text,
                );
            }
            ctx.scene.pop_clip();
        }

        if !self.text.is_empty() {
            let cb = self.clear_rect(rect, &theme);
            if self.hover_clear {
                ctx.scene
                    .fill_rounded_rect(cb, cb.width() / 2.0, colors.shade.scale_alpha(1.8));
            }
            icon::close(ctx, colors.dim_text, cb, 4.0);
        }
    }
}

fn prev_boundary(s: &str, i: usize) -> usize {
    s[..i]
        .char_indices()
        .next_back()
        .map(|(j, _)| j)
        .unwrap_or(0)
}

fn next_boundary(s: &str, i: usize) -> usize {
    s[i..].chars().next().map(|c| i + c.len_utf8()).unwrap_or(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_and_cursor() {
        let mut e = SearchEntry::new("Szukaj");
        e.insert_str("abc");
        assert_eq!(e.text(), "abc");
        assert_eq!(e.cursor(), 3);
        e.move_left();
        e.insert_char('X');
        assert_eq!(e.text(), "abXc");
        e.backspace();
        assert_eq!(e.text(), "abc");
        assert_eq!(e.cursor(), 2);
        e.delete_forward(); // usuwa 'c' za kursorem
        assert_eq!(e.text(), "ab");
        e.home();
        e.delete_forward(); // usuwa 'a'
        assert_eq!(e.text(), "b");
    }

    #[test]
    fn multibyte_is_char_aware() {
        let mut e = SearchEntry::new("");
        e.insert_str("ząb");
        assert_eq!(e.text(), "ząb");
        e.backspace();
        assert_eq!(e.text(), "zą");
        e.move_left();
        e.backspace();
        assert_eq!(e.text(), "ą");
    }

    #[test]
    fn clear_resets_state() {
        let mut e = SearchEntry::new("");
        e.insert_str("x");
        e.clear();
        assert!(e.is_empty());
        assert_eq!(e.cursor(), 0);
    }

    #[test]
    fn clear_hit_only_when_non_empty() {
        let theme = Theme::light();
        let rect = Rect::from_xywh(0.0, 0.0, 200.0, 34.0);
        let mut e = SearchEntry::new("");
        let c = e.clear_rect(rect, &theme).center();
        assert!(!e.clear_at(rect, &theme, c));
        e.insert_str("abc");
        assert!(e.clear_at(rect, &theme, c));
    }

    #[test]
    fn height_from_metrics() {
        let theme = Theme::light();
        assert_eq!(
            SearchEntry::new("").height(&theme),
            theme.metrics.entry.height
        );
    }
}
