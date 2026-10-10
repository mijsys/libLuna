//! # libluna-i18n
//!
//! Lekki system wielojęzyczny dla libLuny — bez gettext i bez zależności zewnętrznych.
//!
//! Model jest celowo prosty, w duchu gettext:
//!
//! - **kluczem jest angielski napis** (np. `"Appearance"`), więc brak tłumaczenia
//!   degraduje się łagodnie do angielskiego;
//! - każdy język to **osobny plik** `i18n/<kod>.lang` (format `klucz = tłumaczenie`),
//!   wbudowany w binarkę przez `include_str!` — łatwo dodawać i edytować;
//! - dodatkowo języki można **doładować w czasie działania** z katalogów
//!   (`/usr/share/luna/i18n`, `~/.local/share/luna/i18n`, `~/.config/luna/i18n`).
//!
//! ```no_run
//! use libluna_i18n::{tr, Locale};
//! libluna_i18n::set_locale(Locale::new("pl"));
//! assert_eq!(tr("Appearance"), "Wygląd");
//! ```
//!
//! Podstawianie argumentów: `tr_args("Hello, {name}", &[("name", "Ania")])`.
//! Każdy katalog może zdefiniować `__name__` — natywną nazwę języka do UI.

use std::collections::{BTreeMap, HashMap};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};

/// Klucz z nazwą języka w danym katalogu (np. `Polski`).
const NAME_KEY: &str = "__name__";

/// Znacznik języka, np. `pl`, `pl_PL`, `en`, `de`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Locale(String);

impl Locale {
    /// Tworzy język z tagu (np. `pl_PL.UTF-8` → `pl`).
    pub fn new(tag: impl AsRef<str>) -> Self {
        let raw = tag.as_ref();
        // Utnij kodowanie (`.UTF-8`) i modyfikator (`@euro`).
        let base = raw.split(['.', '@']).next().unwrap_or(raw);
        let language = base.split(['_', '-']).next().unwrap_or(base);
        Self(language.to_ascii_lowercase())
    }

    /// Wykrywa język ze środowiska (`LC_ALL`, `LC_MESSAGES`, `LANG`).
    pub fn from_env() -> Self {
        for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(value) = std::env::var(key) {
                if !value.is_empty() && value != "C" && value != "POSIX" {
                    return Locale::new(value);
                }
            }
        }
        Locale::new("en")
    }

    /// Kod języka (np. `pl`).
    pub fn language(&self) -> &str {
        &self.0
    }
}

impl Default for Locale {
    fn default() -> Self {
        Locale::new("en")
    }
}

impl From<&str> for Locale {
    fn from(value: &str) -> Self {
        Locale::new(value)
    }
}

impl From<String> for Locale {
    fn from(value: String) -> Self {
        Locale::new(value)
    }
}

/// Katalog tłumaczeń jednego języka.
#[derive(Debug, Clone, Default)]
pub struct Catalog {
    entries: HashMap<String, String>,
}

impl Catalog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Buduje katalog z tekstu (format `klucz = tłumaczenie`).
    pub fn parse(src: &str) -> Self {
        let mut catalog = Self::new();
        catalog.parse_into(src);
        catalog
    }

    /// Dodaje/aktualizuje wpisy z tekstu (pozwala nakładać pliki na siebie).
    pub fn parse_into(&mut self, src: &str) {
        for line in src.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                self.entries
                    .insert(key.trim().to_owned(), value.trim().to_owned());
            }
        }
    }

    /// Tłumaczenie dla klucza, jeśli istnieje.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(String::as_str)
    }

    /// Natywna nazwa języka z `__name__` (jeśli zdefiniowana).
    pub fn name(&self) -> Option<&str> {
        self.get(NAME_KEY)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Zbiór katalogów + bieżący język + język zapasowy.
#[derive(Debug, Clone)]
pub struct Translations {
    locale: Locale,
    catalogs: BTreeMap<String, Catalog>,
    fallback: String,
}

impl Translations {
    /// Tworzy pusty zbiór z językiem zapasowym (domyślnie `en`).
    pub fn new(locale: Locale) -> Self {
        Self {
            locale,
            catalogs: BTreeMap::new(),
            fallback: "en".to_owned(),
        }
    }

    /// Dodaje/nałada katalog języka z tekstu.
    pub fn add_source(&mut self, language: &str, src: &str) -> &mut Self {
        self.catalogs
            .entry(language.to_ascii_lowercase())
            .or_default()
            .parse_into(src);
        self
    }

    /// Zestaw wbudowanych języków libLuny (pliki `i18n/*.lang` wbudowane w binarkę).
    pub fn builtin() -> Self {
        let mut t = Translations::new(Locale::from_env());
        t.add_source("en", include_str!("../i18n/en.lang"));
        t.add_source("pl", include_str!("../i18n/pl.lang"));
        t.add_source("de", include_str!("../i18n/de.lang"));
        t
    }

    pub fn locale(&self) -> &Locale {
        &self.locale
    }

    /// Ustawia bieżący język.
    pub fn set_locale(&mut self, locale: Locale) {
        self.locale = locale;
    }

    /// Krótki kod bieżącego języka.
    pub fn language(&self) -> &str {
        self.locale.language()
    }

    /// Dostępne kody języków (posortowane).
    pub fn languages(&self) -> Vec<String> {
        self.catalogs.keys().cloned().collect()
    }

    /// Natywna nazwa języka z jego katalogu (np. `Polski`).
    pub fn language_name(&self, language: &str) -> Option<&str> {
        self.catalogs.get(language).and_then(Catalog::name)
    }

    /// Tłumaczenie klucza; przy braku wpisu zwraca sam klucz (angielski).
    pub fn translate<'a>(&'a self, key: &'a str) -> &'a str {
        if let Some(catalog) = self.catalogs.get(self.locale.language()) {
            if let Some(value) = catalog.get(key) {
                return value;
            }
        }
        if let Some(catalog) = self.catalogs.get(&self.fallback) {
            if let Some(value) = catalog.get(key) {
                return value;
            }
        }
        key
    }

    /// Tłumaczenie z podstawieniem `{argumentów}`.
    pub fn translate_args(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut out = self.translate(key).to_owned();
        for (name, value) in args {
            out = out.replace(&format!("{{{name}}}"), value);
        }
        out
    }

    /// Doładowuje katalog z pojedynczego pliku `*.lang` (kod języka = nazwa pliku).
    pub fn load_file(&mut self, path: &Path) -> io::Result<()> {
        let Some(code) = path.file_stem().and_then(|s| s.to_str()) else {
            return Ok(());
        };
        let src = std::fs::read_to_string(path)?;
        self.add_source(code, &src);
        Ok(())
    }

    /// Doładowuje wszystkie pliki `*.lang` z katalogu; zwraca liczbę wczytanych.
    pub fn load_dir(&mut self, dir: &Path) -> usize {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return 0;
        };
        let mut count = 0;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("lang")
                && self.load_file(&path).is_ok()
            {
                count += 1;
            }
        }
        count
    }
}

impl Default for Translations {
    fn default() -> Self {
        Translations::builtin()
    }
}

/// Standardowe katalogi wyszukiwania języków doładowywanych w runtime.
pub fn catalog_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    dirs.push(PathBuf::from("/usr/share/luna/i18n"));
    if let Some(data) = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
    {
        dirs.push(data.join("luna/i18n"));
    }
    if let Some(config) = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
    {
        dirs.push(config.join("luna/i18n"));
    }
    dirs
}

/// Globalna instancja tłumaczeń.
fn global() -> &'static RwLock<Translations> {
    static GLOBAL: OnceLock<RwLock<Translations>> = OnceLock::new();
    GLOBAL.get_or_init(|| RwLock::new(Translations::builtin()))
}

/// Ustawia globalny język.
pub fn set_locale(locale: impl Into<Locale>) {
    if let Ok(mut guard) = global().write() {
        guard.set_locale(locale.into());
    }
}

/// Bieżący globalny język.
pub fn locale() -> Locale {
    global()
        .read()
        .map(|g| g.locale().clone())
        .unwrap_or_default()
}

/// Doładowuje języki ze standardowych katalogów (nadpisują wbudowane).
/// Zwraca liczbę wczytanych plików.
pub fn load_catalogs_from_dirs() -> usize {
    let dirs = catalog_dirs();
    match global().write() {
        Ok(mut guard) => dirs.iter().map(|d| guard.load_dir(d)).sum(),
        Err(_) => 0,
    }
}

/// Dostępne kody języków (globalnie).
pub fn languages() -> Vec<String> {
    global().read().map(|g| g.languages()).unwrap_or_default()
}

/// Natywna nazwa języka (globalnie).
pub fn language_name(language: &str) -> Option<String> {
    global()
        .read()
        .ok()
        .and_then(|g| g.language_name(language).map(str::to_owned))
}

/// Przetłumaczony napis (globalnie).
pub fn tr(key: &str) -> String {
    global()
        .read()
        .map(|g| g.translate(key).to_owned())
        .unwrap_or_else(|_| key.to_owned())
}

/// Przetłumaczony napis z podstawieniem argumentów (globalnie).
pub fn tr_args(key: &str, args: &[(&str, &str)]) -> String {
    global()
        .read()
        .map(|g| g.translate_args(key, args))
        .unwrap_or_else(|_| key.to_owned())
}

/// Skrót do [`tr`] / [`tr_args`].
///
/// ```ignore
/// let s = t!("Appearance");
/// let s = t!("Witaj, {name}", name = "Ania");
/// ```
#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::tr($key)
    };
    ($key:expr, $($k:ident = $v:expr),+ $(,)?) => {
        $crate::tr_args($key, &[$( (stringify!($k), $v) ),+])
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn locale_normalization() {
        assert_eq!(Locale::new("pl_PL.UTF-8").language(), "pl");
        assert_eq!(Locale::new("de-DE").language(), "de");
        assert_eq!(Locale::new("en_US").language(), "en");
        assert_eq!(Locale::new("C").language(), "c");
    }

    #[test]
    fn catalog_parses_and_ignores_comments() {
        let c = Catalog::parse("# komentarz\nA = B\n\nC = D = E\n");
        assert_eq!(c.get("A"), Some("B"));
        // Dzielimy tylko po pierwszym `=`.
        assert_eq!(c.get("C"), Some("D = E"));
    }

    #[test]
    fn builtin_translates_and_falls_back() {
        let mut t = Translations::new(Locale::new("pl"));
        t.add_source("pl", include_str!("../i18n/pl.lang"));
        assert_eq!(t.translate("Appearance"), "Wygląd");
        assert_eq!(t.translate("Nieznany klucz"), "Nieznany klucz");
    }

    #[test]
    fn switching_locale_changes_output() {
        let mut t = Translations::new(Locale::new("en"));
        t.add_source("pl", include_str!("../i18n/pl.lang"));
        t.add_source("de", include_str!("../i18n/de.lang"));
        t.set_locale(Locale::new("de"));
        assert_eq!(t.translate("Dark"), "Dunkel");
        t.set_locale(Locale::new("pl"));
        assert_eq!(t.translate("Dark"), "Ciemny");
    }

    #[test]
    fn language_names_and_listing() {
        let t = Translations::builtin();
        assert_eq!(t.language_name("pl"), Some("Polski"));
        assert_eq!(t.language_name("de"), Some("Deutsch"));
        assert_eq!(t.language_name("en"), Some("English"));
        assert!(t.languages().contains(&"pl".to_owned()));
    }

    #[test]
    fn args_substitution() {
        let t = Translations::new(Locale::new("en"));
        assert_eq!(
            t.translate_args("Hi, {name}!", &[("name", "Ada")]),
            "Hi, Ada!"
        );
    }

    #[test]
    fn load_file_overrides_and_adds() {
        let dir = std::env::temp_dir().join("libluna-i18n-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("xx.lang");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "__name__ = Testish").unwrap();
        writeln!(f, "Appearance = Xyglad").unwrap();
        drop(f);

        let mut t = Translations::builtin();
        t.load_file(&path).unwrap();
        t.set_locale(Locale::new("xx"));
        assert_eq!(t.translate("Appearance"), "Xyglad");
        assert_eq!(t.language_name("xx"), Some("Testish"));
        // Klucz nieprzetłumaczony w `xx` spada do bazy `en`.
        assert_eq!(t.translate("System"), "System");

        let _ = std::fs::remove_file(&path);
    }
}
