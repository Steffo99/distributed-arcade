use axum::http::{HeaderMap, StatusCode};
use regex::Regex;
use crate::outcome;
use crate::utils::token::SecureToken;


pub trait Authorize<'h> {
    fn get_authorization_or_401(&'h self, scheme: &str) -> Result<&'h str, outcome::RequestTuple>;
}

impl<'h> Authorize<'h> for HeaderMap {
    fn get_authorization_or_401(&'h self, scheme: &str) -> Result<&'h str, outcome::RequestTuple> {
        log::trace!("Searching for the {scheme:?} Authorization header...");

        log::trace!("Compiling regex...");
        let auth_header_regex = Regex::new(&*format!(r#"{scheme} (\S+)"#))
            .expect("scheme to create a valid regex");

        log::trace!("Searching Authorization header...");
        let token = self.get("Authorization")
            .ok_or_else(|| (StatusCode::UNAUTHORIZED, outcome::req_error!("Missing Authorization header")))?;

        log::trace!("Converting Authorization header to ASCII string...");
        let token = token.to_str()
            .map_err(|_| (StatusCode::UNAUTHORIZED, outcome::req_error!("Malformed Authorization header")))?;

        log::trace!("Capturing the Authorization scheme value...");
        let token = auth_header_regex.captures(token)
            .ok_or_else(|| (StatusCode::UNAUTHORIZED, outcome::req_error!("Malformed Authorization header")))?;

        log::trace!("Getting the Authorization scheme match...");
        let token = token.get(1)
            .ok_or_else(|| (StatusCode::UNAUTHORIZED, outcome::req_error!("Invalid Authorization header")))?;

        log::trace!("Obtained Authorization scheme token!");
        Ok(token.as_str())
    }
}


pub trait Generate where Self: Sized {
    fn new_or_500() -> Result<Self, outcome::RequestTuple>;
}

impl Generate for SecureToken {
    fn new_or_500() -> Result<Self, outcome::RequestTuple> {
        SecureToken::new()
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, outcome::req_error!("Could not generate token")))
    }
}