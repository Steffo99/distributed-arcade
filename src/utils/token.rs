//! Module defining and implementing [`SecureToken`].

use serde::Serialize;
use serde::Deserialize;

/// Alphabet for base-62 encoding.
const TOKEN_CHARS: &[char; 62] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'
];

/// A cryptographically secure, [base-62](TOKEN_CHARS) token.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecureToken(pub String);

impl SecureToken {
    pub fn new() -> Result<Self, rand::Error> {
        log::trace!("Initializing secure RNG...");
        let mut rng = rand::rngs::OsRng::default();

        log::trace!("Generating a secure token...");
        let mut token: [u32; 16] = [0; 16];
        rand::Fill::try_fill(&mut token, &mut rng)?;

        let token = token.iter()
            .map(|e|
                // Only works on platforms where usize >= 32-bit?
                TOKEN_CHARS.get(*e as usize % 62)
                    .expect("randomly generated value to be a valid index")
            )
            .collect::<String>();

        Ok(Self(token))
    }
}
