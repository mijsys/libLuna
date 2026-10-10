//! # libluna-platform
//!
//! Warstwa platformy libLuny — odpowiednik GDK w GTK.
//!
//! Odpowiada za połączenie z kompozytorem Wayland (Selene), tworzenie okien przez
//! `xdg-shell` z dekoracjami po stronie klienta (CSD), bufory `wl_shm`, obsługę
//! wejścia (`seat`, `xkbcommon`) oraz pętlę zdarzeń.
//!
//! Punkt wejścia to [`run`] z [`AppConfig`]: otwiera okno i wywołuje domknięcie
//! rysujące przy każdym odświeżeniu.

pub mod window;

pub use window::{run, AppConfig, Frame};
