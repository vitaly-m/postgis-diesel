//! Error enumeration which may occur during the creation of geometries.

#[derive(Debug)]
/// Enum representing errors that can occur when reading geometries from SQL.
pub enum Error {
    /// An error occurred while reading the geometry from SQL.
    IncompatibleSpatialReferenceSystemIdentifier {
        /// The expected SRID.
        expected: Option<u32>,
        /// The actual SRID.
        actual: Option<u32>,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IncompatibleSpatialReferenceSystemIdentifier { expected, actual } => {
                write!(
                    f,
                    "Incompatible SRID: expected {:?}, actual {:?}",
                    expected, actual
                )
            }
        }
    }
}

impl std::error::Error for Error {}
