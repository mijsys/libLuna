//! Zdarzenia wejścia i okna — wspólny format dla całej reszty toolkitu.
//!
//! Warstwa platformy tłumaczy zdarzenia Waylanda na [`Event`]; widgety konsumują
//! je w jednym drzewie i zwracają, czy zdarzenie zostało obsłużone.

use crate::geometry::{Point, Size};

/// Klawisz myszy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButton {
    Left,
    Middle,
    Right,
    Back,
    Forward,
    Other(u32),
}

/// Stan wciśnięcia (dla przycisków i klawiszy).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Pressed,
    Released,
}

/// Modyfikatory klawiatury.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub logo: bool,
}

impl Modifiers {
    pub const NONE: Modifiers = Modifiers {
        ctrl: false,
        alt: false,
        shift: false,
        logo: false,
    };
}

/// Zdarzenie przycisku wskaźnika.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerEvent {
    pub position: Point,
    pub button: PointerButton,
    pub state: ButtonState,
    pub modifiers: Modifiers,
}

/// Zdarzenie przewijania (rolka/gładzik); wartości w „krokach”.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ScrollEvent {
    pub position: Point,
    pub dx: f64,
    pub dy: f64,
}

/// Zdarzenie klawiatury.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyEvent {
    /// Keysym w rozumieniu XKB.
    pub keysym: u32,
    /// Tekst wygenerowany przez klawisz (jeśli dotyczy).
    pub text: Option<String>,
    pub state: ButtonState,
    pub modifiers: Modifiers,
    /// Czy zdarzenie zostało powtórzone przez auto-repeat.
    pub repeat: bool,
}

/// Zdarzenia cyklu życia okna.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowEvent {
    /// Nowy rozmiar w pikselach logicznych.
    Resize(Size),
    /// Skala ekranu uległa zmianie.
    ScaleChanged(f64),
    /// Uzyskano/utracono fokus.
    Focused(bool),
    /// Użytkownik (lub kompozytor) poprosił o zamknięcie okna.
    CloseRequested,
    /// Należy przerysować zawartość.
    Redraw,
}

/// Zunifikowane zdarzenie trafiające do drzewa widgetów.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    PointerButton(PointerEvent),
    /// Ruch wskaźnika; `position` w pikselach logicznych.
    PointerMotion {
        position: Point,
        modifiers: Modifiers,
    },
    /// Wskaźnik opuścił powierzchnię.
    PointerLeave,
    Scroll(ScrollEvent),
    Key(KeyEvent),
    /// Wprowadzony tekst (po złożeniu IME).
    Text(String),
    Window(WindowEvent),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_event_is_copy() {
        let e = PointerEvent {
            position: Point::new(1.0, 2.0),
            button: PointerButton::Left,
            state: ButtonState::Pressed,
            modifiers: Modifiers::NONE,
        };
        let copy = e;
        assert_eq!(e, copy);
    }
}
