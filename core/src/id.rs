use {
    rand::{rngs::StdRng, RngCore, SeedableRng},
    std::{
        fmt::{Debug, Display, Formatter, Result as FmtResult},
        io::Result as IoResult,
    },
    tokio::io::{AsyncRead, AsyncReadExt},
};

/// The length of identifiers.
pub const ID_LENGTH: usize = 16;

/// Lucene identifiers.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id {
    id: [u8; ID_LENGTH],
}

impl Debug for Id {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "Id({:#x?})", self.id)
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        for b in self.id {
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
}

impl Id {
    /// Generate a new random id.
    ///
    /// Note: I believe the comments in the Lucene Java implementation to be incorrect for Rust's `rand` crate, so
    /// we go ahead and use it (and a cryptographically secure RNG, even though it is overkill) here.
    pub fn random_id() -> Self {
        let mut id = [0u8; ID_LENGTH];
        StdRng::from_entropy().fill_bytes(&mut id);
        Self {
            id,
        }
    }

    /// Read an id from a stream. Returns the id.
    pub async fn read_from<R: AsyncRead + Unpin>(r: &mut R) -> IoResult<Self> {
        let mut id = [0u8; ID_LENGTH];
        r.read_exact(&mut id).await?;
        Ok(Self {
            id,
        })
    }
}
