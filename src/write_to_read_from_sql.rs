//! Submodule defining the WriteToSql ReadFromSql private traits.

#[cfg(feature = "diesel")]
pub(crate) trait WriteToSql {
    fn write_to_sql<W>(&self, out: &mut W) -> diesel::serialize::Result
    where
        W: std::io::Write;
}

#[cfg(feature = "diesel")]
pub(crate) trait ReadFromSql: Sized {
    fn read_from_sql(bytes: &[u8]) -> diesel::deserialize::Result<Self>;
}
