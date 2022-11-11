//! Module containing various types recurring in multiple modules of the crate.


use serde::Serialize;
use serde::Deserialize;


/// A sorting order for scores.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum SortingOrder {
    /// The greater the score, the worse it is.
    #[serde(rename = "ASC")]
    Ascending,
    /// The greater the score, the better it is.
    #[serde(rename = "DSC")]
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
impl From<SortingOrder> for String {
    fn from(ord: SortingOrder) -> Self {
        match ord {
            SortingOrder::Ascending  => "ASC".to_string(),
            SortingOrder::Descending => "DSC".to_string(),
        }
    }
}

/// How the [`SortingOrder`] is retrieved from [Redis].
impl TryFrom<String> for SortingOrder {
    type Error = ();

    fn try_from(val: String) -> Result<Self, Self::Error> {
        match val.as_str() {
            "ASC" => Ok(Self::Ascending),
            "DSC" => Ok(Self::Descending),
            _ => Err(())
        }
    }
}
