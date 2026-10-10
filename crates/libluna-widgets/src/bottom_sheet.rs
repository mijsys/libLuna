//! `BottomSheet` — dolny panel (modalny), wysuwany z dołu okna.
//!
//! Widget immediate: trzyma stan otwarcia, opcjonalny tytuł i przesunięcie przeciągania.
//! Aplikacja woła [`BottomSheet::paint`] oraz metody trafień ([`BottomSheet::close_at`],
//! [`BottomSheet::scrim_at`]) i obsługę przeciągania
//! ([`BottomSheet::begin_drag`] / [`BottomSheet::update_drag`] / [`BottomSheet::end_drag`]).
//! Zawartość panelu rysuje aplikacja w [`BottomSheet::content_rect`].

use libluna_core::style::weight;
use libluna_core::{Color, Point, Rect, Size, Theme};

use crate::ctx::PaintCtx;
use crate::icon;

/// Bok obszaru przycisku zamknięcia.
const CLOSE: f64 = 24.0;
/// Odstęp „grabbera” od górnej krawędzi.
const HANDLE_TOP: f64 = 8.0;

/// Dolny panel.
#[derive(Debug, Clone, Default)]
pub struct BottomSheet {
    title: Option<String>,
    open: bool,
    height: Option<f64>,
    /// Bieżące przesunięcie w dół podczas przeciągania (px).
    drag: f64,
    /// Pozycja wskaźnika (y) na początku przeciągania.
    drag_from: Option<f64>,
    hover_close: bool,
}

impl BottomSheet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    /// Wysokość panelu (domyślnie z tokenów motywu).
    pub fn set_height(&mut self, height: f64) {
        self.height = Some(height);
    }

    pub fn set_open(&mut self, open: bool) {
        self.open = open;
        if !open {
            self.drag = 0.0;
            self.drag_from = None;
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn set_hover_close(&mut self, hovered: bool) {
        self.hover_close = hovered;
    }

    pub fn height(&self, theme: &Theme) -> f64 {
        self.height.unwrap_or(theme.metrics.sheet.height)
    }

    /// Prostokąt panelu (przyklejony do dołu, z uwzględnieniem przeciągania).
    pub fn sheet_rect(&self, size: Size, theme: &Theme) -> Rect {
        let h = self.height(theme);
        Rect::from_xywh(0.0, size.height - h + self.drag, size.width, h)
    }

    /// Pełnoekranowe tło przygaszające.
    pub fn scrim_rect(&self, size: Size) -> Rect {
        Rect::from_xywh(0.0, 0.0, size.width, size.height)
    }

    /// Strefa chwytu do przeciągania (nagłówek z „grabberem”).
    pub fn drag_zone(&self, size: Size, theme: &Theme) -> Rect {
        let s = self.sheet_rect(size, theme);
        Rect::from_xywh(s.x(), s.y(), s.width(), theme.metrics.sheet.header)
    }

    /// Prostokąt „grabbera”.
    pub fn grabber_rect(&self, size: Size, theme: &Theme) -> Rect {
        let s = self.sheet_rect(size, theme);
        let m = theme.metrics.sheet;
        Rect::from_xywh(
            s.center().x - m.handle_width / 2.0,
            s.y() + HANDLE_TOP,
            m.handle_width,
            m.handle_height,
        )
    }

    /// Prostokąt przycisku zamknięcia.
    pub fn close_rect(&self, size: Size, theme: &Theme) -> Rect {
        let s = self.sheet_rect(size, theme);
        let md = theme.spacing.md;
        let cy = s.y() + theme.metrics.sheet.header / 2.0;
        Rect::from_xywh(s.right() - md - CLOSE, cy - CLOSE / 2.0, CLOSE, CLOSE)
    }

    /// Obszar zawartości (poniżej nagłówka).
    pub fn content_rect(&self, size: Size, theme: &Theme) -> Rect {
        let s = self.sheet_rect(size, theme);
        let header = theme.metrics.sheet.header;
        Rect::from_xywh(
            s.x(),
            s.y() + header,
            s.width(),
            (s.height() - header).max(0.0),
        )
    }

    /// Czy punkt trafia w przycisk zamknięcia.
    pub fn close_at(&self, size: Size, theme: &Theme, p: Point) -> bool {
        self.open && self.close_rect(size, theme).contains(p)
    }

    /// Czy punkt trafia w tło poza panelem (klik zamykający).
    pub fn scrim_at(&self, size: Size, theme: &Theme, p: Point) -> bool {
        self.open && !self.sheet_rect(size, theme).contains(p)
    }

    /// Czy punkt trafia w strefę przeciągania panelu.
    pub fn drag_zone_at(&self, size: Size, theme: &Theme, p: Point) -> bool {
        self.open && self.drag_zone(size, theme).contains(p)
    }

    /// Rozpoczyna przeciąganie od pozycji wskaźnika `y`.
    pub fn begin_drag(&mut self, y: f64) {
        self.drag_from = Some(y);
    }

    /// Aktualizuje przeciągnięcie (w dół).
    pub fn update_drag(&mut self, y: f64) {
        if let Some(from) = self.drag_from {
            self.drag = (y - from).max(0.0);
        }
    }

    /// Kończy przeciąganie; zwraca `true`, jeśli panel został zamknięty.
    pub fn end_drag(&mut self, theme: &Theme) -> bool {
        let dismissed = self.drag > theme.metrics.sheet.dismiss;
        if dismissed {
            self.open = false;
        }
        self.drag = 0.0;
        self.drag_from = None;
        dismissed
    }

    /// Rysuje przygaszenie i panel (na wierzchu sceny).
    pub fn paint(&self, ctx: &mut PaintCtx, size: Size) {
        if !self.open {
            return;
        }
        let theme = *ctx.theme;
        let m = theme.metrics.sheet;
        let colors = theme.colors;

        ctx.scene.fill_rect(
            self.scrim_rect(size),
            Color::rgba(0.0, 0.0, 0.0, m.scrim as f32),
        );

        let sheet = self.sheet_rect(size, &theme);
        // Panel: zaokrąglone górne narożniki, dolne „przycięte” do krawędzi.
        ctx.scene.fill_rounded_rect(sheet, m.radius, colors.card_bg);
        ctx.scene.fill_rect(
            Rect::from_xywh(
                sheet.x(),
                sheet.y() + m.radius,
                sheet.width(),
                (sheet.height() - m.radius).max(0.0),
            ),
            colors.card_bg,
        );
        ctx.scene.fill_rect(
            Rect::from_xywh(sheet.x(), sheet.y(), sheet.width(), 1.0),
            colors.border,
        );

        let g = self.grabber_rect(size, &theme);
        ctx.scene
            .fill_rounded_rect(g, g.height() / 2.0, colors.dim_text.scale_alpha(0.5));

        if let Some(title) = &self.title {
            ctx.label_center(
                sheet.center().x,
                sheet.y() + m.header / 2.0,
                title,
                theme.typography.body.size,
                weight::BOLD,
                colors.text,
            );
        }

        let c = self.close_rect(size, &theme);
        if self.hover_close {
            ctx.scene
                .fill_rounded_rect(c, theme.radius.small, colors.shade);
        }
        icon::close(ctx, colors.dim_text, c, 5.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn size() -> Size {
        Size::new(900.0, 820.0)
    }

    #[test]
    fn sheet_is_bottom_anchored() {
        let theme = Theme::light();
        let mut s = BottomSheet::new();
        s.set_open(true);
        let r = s.sheet_rect(size(), &theme);
        assert!((r.bottom() - 820.0).abs() < 1e-9);
        assert!((r.height() - theme.metrics.sheet.height).abs() < 1e-9);
    }

    #[test]
    fn scrim_hit_is_above_sheet_only() {
        let theme = Theme::light();
        let mut s = BottomSheet::new();
        s.set_open(true);
        let sheet = s.sheet_rect(size(), &theme);
        assert!(s.scrim_at(size(), &theme, Point::new(450.0, sheet.y() - 20.0)));
        assert!(!s.scrim_at(size(), &theme, Point::new(450.0, sheet.y() + 20.0)));
    }

    #[test]
    fn close_hit_and_content_below_header() {
        let theme = Theme::light();
        let mut s = BottomSheet::new().title("O aplikacji");
        s.set_open(true);
        assert!(s.close_at(size(), &theme, s.close_rect(size(), &theme).center()));
        let c = s.content_rect(size(), &theme);
        assert!(c.y() >= s.sheet_rect(size(), &theme).y() + theme.metrics.sheet.header);
    }

    #[test]
    fn closed_sheet_ignores_hits() {
        let theme = Theme::light();
        let s = BottomSheet::new();
        assert!(!s.scrim_at(size(), &theme, Point::new(10.0, 10.0)));
        assert!(!s.close_at(size(), &theme, Point::new(880.0, 520.0)));
    }

    #[test]
    fn drag_past_threshold_dismisses() {
        let theme = Theme::light();
        let mut s = BottomSheet::new();
        s.set_open(true);
        s.begin_drag(500.0);
        s.update_drag(500.0 + theme.metrics.sheet.dismiss + 5.0);
        assert!(s.end_drag(&theme));
        assert!(!s.is_open());
    }

    #[test]
    fn small_drag_keeps_open() {
        let theme = Theme::light();
        let mut s = BottomSheet::new();
        s.set_open(true);
        s.begin_drag(500.0);
        s.update_drag(500.0 + 10.0);
        assert!(!s.end_drag(&theme));
        assert!(s.is_open());
    }
}
