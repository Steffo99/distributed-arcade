//! Module defining and implementing [`SortingOrder`].

use serde::Serialize;
use serde::Deserialize;


/// A sorting order for scores.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum SortingOrder {
    /// The greater the score, the worse it is.
    Ascending,

    /// The greater the score, the better it is.
    Descending,
}

impl SortingOrder {
    /// Get the mode to use when using the [Redis] command [`ZADD`](https://redis.io/commands/zadd/).
    pub fn zadd_mode(&self) -> String {
        match self {
            Self::Ascending => "LT".to_string(),
            Self::Descending => "GT".to_string(),
        }
    }
}

/// How the [`SortingOrder`] is stored in [Redis].
impl From<SortingOrder> for &str {
    fn from(ord: SortingOrder) -> Self {
        match ord {
            SortingOrder::Ascending  => "Ascending",
            SortingOrder::Descending => "Descending",
        }
    }
}

/// How the [`SortingOrder`] is retrieved from [Redis].
impl TryFrom<&str> for SortingOrder {
    type Error = ();

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val {
            "Ascending"  => Ok(Self::Ascending),
            "Descending" => Ok(Self::Descending),
            _ => Err(())
        }
    }
}
