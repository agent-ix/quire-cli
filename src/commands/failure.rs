//! Typed domain outcome classification for the CLI process boundary.

use ix_cli_kit::exit::Outcome;
use quire_cli::safety::PathError;

#[derive(Debug)]
pub(crate) struct Failure {
    outcome: Outcome,
    error: anyhow::Error,
}

impl std::fmt::Display for Failure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.error, formatter)
    }
}

impl std::error::Error for Failure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.error.source()
    }
}

pub(crate) fn classified(outcome: Outcome, error: anyhow::Error) -> anyhow::Error {
    Failure { outcome, error }.into()
}

pub(crate) fn invalid(error: impl Into<anyhow::Error>) -> anyhow::Error {
    classified(Outcome::Invalid, error.into())
}

pub(crate) fn refused(error: impl Into<anyhow::Error>) -> anyhow::Error {
    classified(Outcome::Refused, error.into())
}

pub(crate) fn input(error: anyhow::Error) -> anyhow::Error {
    if error.chain().any(|cause| cause.is::<PathError>()) {
        return error;
    }
    let outcome = error
        .chain()
        .find_map(|cause| cause.downcast_ref::<std::io::Error>())
        .map_or(Outcome::Invalid, |source| match source.kind() {
            std::io::ErrorKind::NotFound
            | std::io::ErrorKind::InvalidInput
            | std::io::ErrorKind::InvalidData => Outcome::Invalid,
            std::io::ErrorKind::PermissionDenied => Outcome::Refused,
            _ => Outcome::Internal,
        });
    classified(outcome, error)
}

pub(crate) fn outcome(error: &anyhow::Error) -> Outcome {
    for cause in error.chain() {
        if let Some(failure) = cause.downcast_ref::<Failure>() {
            return failure.outcome;
        }
        if let Some(path) = cause.downcast_ref::<PathError>() {
            return match path {
                PathError::Traversal(_) => Outcome::Refused,
                PathError::NotADirectory(_) | PathError::NotAFile(_) => Outcome::Invalid,
                PathError::Io { source, .. } => match source.kind() {
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::InvalidInput => {
                        Outcome::Invalid
                    }
                    std::io::ErrorKind::PermissionDenied => Outcome::Refused,
                    _ => Outcome::Internal,
                },
            };
        }
        if cause.is::<super::DocumentRootError>() {
            return Outcome::Invalid;
        }
    }
    Outcome::Internal
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

    #[test]
    #[trace("FR-007-AC-8")]
    fn diagnostic_wording_does_not_classify_the_outcome() {
        for message in ["invalid refusal internal", "fictional diagnostic"] {
            for expected in [Outcome::Invalid, Outcome::Refused, Outcome::Partial] {
                let error =
                    classified(expected, anyhow::anyhow!(message)).context("request context");
                assert_eq!(outcome(&error), expected);
                assert_eq!(format!("{error:#}"), format!("request context: {message}"));
            }
            assert_eq!(outcome(&anyhow::anyhow!(message)), Outcome::Internal);
        }
    }

    #[test]
    #[trace("FR-007-AC-2", "FR-007-AC-3", "FR-007-AC-8")]
    fn input_io_conditions_have_typed_distinct_outcomes() {
        for (kind, expected) in [
            (std::io::ErrorKind::NotFound, Outcome::Invalid),
            (std::io::ErrorKind::InvalidData, Outcome::Invalid),
            (std::io::ErrorKind::PermissionDenied, Outcome::Refused),
            (std::io::ErrorKind::Other, Outcome::Internal),
        ] {
            let source = std::io::Error::new(kind, "fictional input");
            assert_eq!(outcome(&input(source.into())), expected);
        }
    }
}
