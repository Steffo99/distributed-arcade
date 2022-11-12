//! Module defining and implementing the [`Skewer`] trait.

use regex::Regex;


/// Trait to skewer strings into `UPPER-KEBAB-CASE` or `lower-kebab-case`.
pub trait Skewer {
    /// Replace the non-alphanumeric characters of the string with dashes.
    fn to_kebab_anycase(&self) -> String;
    /// Lowercase the string, then [kebabify](to_kebab_anycase) it.
    fn to_kebab_lowercase(&self) -> String;
    /// Uppercase the string, then [kebabify](to_kebab_anycase) it.
    fn to_kebab_uppercase(&self) -> String;
}

impl Skewer for &str {
    fn to_kebab_anycase(&self) -> String {
        lazy_static::lazy_static! {
            static ref INVALID_CHARACTERS_REGEX: Regex = Regex::new(r#"[^A-Za-z0-9-]"#)
                .expect("INVALID_CHARACTERS_REGEX to be valid");
        }

        log::trace!("Kebab-ifying: {self:?}");
        let kebab = INVALID_CHARACTERS_REGEX.replace_all(self, "-").into_owned();
        log::trace!("Kebab-ification complete: {kebab:?}");

        kebab
    }

    fn to_kebab_lowercase(&self) -> String {
        log::trace!("Kebab-i-lower-fying: {self:?}");
        let kebab = self.to_ascii_lowercase().as_str().to_kebab_anycase();
        log::trace!("Kebab-i-lower-ification complete: {kebab:?}");

        kebab
    }

    fn to_kebab_uppercase(&self) -> String {
        log::trace!("Kebab-i-lower-fying: {self:?}");
        let kebab = self.to_ascii_uppercase().as_str().to_kebab_anycase();
        log::trace!("Kebab-i-lower-ification complete: {kebab:?}");

        kebab
    }
}

impl Skewer for String {
    fn to_kebab_anycase(&self) -> String {
        self.as_str().to_kebab_anycase()
    }

    fn to_kebab_lowercase(&self) -> String {
        self.as_str().to_kebab_lowercase()
    }

    fn to_kebab_uppercase(&self) -> String {
        self.as_str().to_kebab_uppercase()
    }
}
