//! Trust stages from the Sweet Potato control room, and what each one lets a
//! builder do. This is the starter's sample domain: replace it with the
//! project's own, keeping the shape of an enum per state and parsing at the edge.

use std::error::Error;
use std::fmt;
use std::str::FromStr;

/// A project's registered trust stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustStage {
    Unverified,
    Supervised,
    Proven,
}

/// What a builder may do at a trust stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuilderAuthority {
    NoBuilds,
    WatchedTicket,
    AsyncTicket,
}

impl TrustStage {
    /// Returns what a builder may do at this stage.
    ///
    /// ```
    /// use rust_pilot::{BuilderAuthority, TrustStage};
    ///
    /// let stage: TrustStage = "stage-1-supervised".parse()?;
    /// assert_eq!(stage.builder_authority(), BuilderAuthority::WatchedTicket);
    /// # Ok::<(), rust_pilot::UnknownStage>(())
    /// ```
    #[must_use]
    pub const fn builder_authority(self) -> BuilderAuthority {
        match self {
            Self::Unverified => BuilderAuthority::NoBuilds,
            Self::Supervised => BuilderAuthority::WatchedTicket,
            Self::Proven => BuilderAuthority::AsyncTicket,
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Unverified => "stage-0-unverified",
            Self::Supervised => "stage-1-supervised",
            Self::Proven => "stage-2-proven",
        }
    }
}

impl FromStr for TrustStage {
    type Err = UnknownStage;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "stage-0-unverified" => Ok(Self::Unverified),
            "stage-1-supervised" => Ok(Self::Supervised),
            "stage-2-proven" => Ok(Self::Proven),
            other => Err(UnknownStage(other.to_owned())),
        }
    }
}

impl fmt::Display for TrustStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl fmt::Display for BuilderAuthority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NoBuilds => "no builder runs",
            Self::WatchedTicket => "one builder runs while Et inspects every step",
            Self::AsyncTicket => "one builder finishes a ticket on its own; Et still merges",
        })
    }
}

/// A trust stage name the control room doesn't define.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownStage(String);

impl fmt::Display for UnknownStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown trust stage {:?}; use stage-0-unverified, stage-1-supervised or stage-2-proven",
            self.0
        )
    }
}

impl Error for UnknownStage {}
