//! Validator for the cache.
//!
//! # Example
//!
//! ```
//! # use rscache::Cache;
//! use rscache::cksm::{ Checksum, OsrsEncode };
//!
//! # fn main() -> rscache::Result<()> {
//! # let cache = Cache::new("./data/osrs_cache")?;
//! let checksum = cache.create_checksum()?;
//!
//! // Encode the checksum with the OSRS protocol.
//! let buffer = checksum.encode()?;
//! # Ok(())
//! # }
//! ```

use std::slice::{Iter, IterMut};

use crate::{codec, codec::Compression};

#[cfg(feature = "rs3")]
use num_bigint::{BigInt, Sign};
#[cfg(feature = "serde-derive")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "rs3")]
use whirlpool::{Digest, Whirlpool};

/// Consumes the `Checksum` and encodes it into a byte buffer
/// using the OSRS protocol.
///
/// After encoding the checksum it can be sent to the client.
///
/// # Errors
///
/// Returns a `CacheError` if the encoding fails.
///
/// # Examples
///
/// ```
/// # use std::net::TcpStream;
/// # use std::io::Write;
/// use rscache::cksm::{ Checksum, OsrsEncode };
///
/// fn encode_checksum(checksum: Checksum, stream: &mut TcpStream) -> rscache::Result<()> {
///     let buffer = checksum.encode()?;
///
///     stream.write_all(&buffer)?;
///     Ok(())
/// }
/// ```
pub trait OsrsEncode {
    fn encode(self) -> crate::Result<Vec<u8>>;
}

/// Consumes the `Checksum` and encodes it into a byte buffer
/// using the RS3 protocol.
///
/// Note: RS3 clients use RSA. The encoding process requires an exponent
/// and a modulus to encode the buffer properly.
///
/// After encoding the checksum it can be sent to the client.
///
/// # Errors
///
/// Returns a `CacheError` if the encoding fails.
///
/// # Examples
///
/// ```
/// # use std::net::TcpStream;
/// # use std::io::Write;
/// # mod env {
/// # pub const EXPONENT: &'static [u8] = b"5206580307236375668350588432916871591810765290737810323990754121164270399789630501436083337726278206128394461017374810549461689174118305784406140446740993";
/// # pub const MODULUS: &'static [u8] = b"6950273013450460376345707589939362735767433035117300645755821424559380572176824658371246045200577956729474374073582306250298535718024104420271215590565201";
/// # }
/// use rscache::cksm::{ Checksum, Rs3Encode };
///
/// fn encode_checksum(checksum: Checksum, stream: &mut TcpStream) -> rscache::Result<()> {
///     let buffer = checksum.encode(env::EXPONENT, env::MODULUS)?;
///
///     stream.write_all(&buffer)?;
///     Ok(())
/// }
/// ```
#[cfg(feature = "rs3")]
pub trait Rs3Encode {
    fn encode(self, exponent: &[u8], modulus: &[u8]) -> crate::Result<Vec<u8>>;
}

/// Contains index validation data.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde-derive", derive(Serialize, Deserialize))]
pub struct Entry {
    pub crc: u32,
    pub version: u32,
    #[cfg(feature = "rs3")]
    pub hash: Vec<u8>,
}

/// Validator for the `Cache`.
///
/// Used to validate cache index files. It contains a list of entries, one entry for each index file.
///
/// In order to create the `Checksum` the
/// [create_checksum()](../struct.Cache.html#method.create_checksum) function has to be
/// called on `Cache`.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[cfg_attr(feature = "serde-derive", derive(Serialize, Deserialize))]
pub struct Checksum {
    index_count: usize,
    entries: Vec<Entry>,
}

impl Checksum {
    pub(crate) fn new(index_count: usize) -> Self {
        Self {
            index_count,
            entries: Vec::with_capacity(index_count),
        }
    }

    pub(crate) fn push(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

    /// Validates crcs with internal crcs.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rscache::Cache;
    /// # fn main() -> rscache::Result<()> {
    /// # let cache = Cache::new("./data/osrs_cache")?;
    /// # let checksum = cache.create_checksum()?;
    /// // client crcs:
    /// let crcs = vec![1593884597, 1029608590, 16840364, 4209099954, 3716821437, 165713182, 686540367,
    ///                 4262755489, 2208636505, 3047082366, 586413816, 2890424900, 3411535427, 3178880569,
    ///                 153718440, 3849392898, 3628627685, 2813112885, 1461700456, 2751169400, 2927815226];
    ///
    /// let valid = checksum.validate(&crcs);
    ///
    /// assert!(valid);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn validate(&self, crcs: &[u32]) -> bool {
        let internal: Vec<u32> = self.entries.iter().map(|entry| entry.crc).collect();
        internal == crcs
    }

    #[inline]
    pub const fn index_count(&self) -> usize {
        self.index_count
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, Entry> {
        self.entries.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, Entry> {
        self.entries.iter_mut()
    }
}

impl OsrsEncode for Checksum {
    #[inline]
    fn encode(self) -> crate::Result<Vec<u8>> {
        let mut buffer = Vec::with_capacity(self.entries.len() * 8);

        for entry in self.entries {
            buffer.extend(&u32::to_be_bytes(entry.crc));
            buffer.extend(&u32::to_be_bytes(entry.version));
        }

        codec::encode(Compression::None, &buffer, None)
    }
}

#[cfg(feature = "rs3")]
impl Rs3Encode for Checksum {
    #[inline]
    fn encode(self, exponent: &[u8], modulus: &[u8]) -> crate::Result<Vec<u8>> {
        let index_count = self.index_count - 1;
        let mut buffer = vec![0; 81 * index_count];

        buffer[0] = index_count as u8;
        for (index, entry) in self.entries.iter().enumerate() {
            let offset = index * 80;
            buffer[offset + 1..=offset + 4].copy_from_slice(&u32::to_be_bytes(entry.crc));
            buffer[offset + 5..=offset + 8].copy_from_slice(&u32::to_be_bytes(entry.version));
            buffer[offset + 9..=offset + 12].copy_from_slice(&u32::to_be_bytes(0));
            buffer[offset + 13..=offset + 16].copy_from_slice(&u32::to_be_bytes(0));
            buffer[offset + 17..=offset + 80].copy_from_slice(&entry.hash);
        }

        let mut hasher = Whirlpool::new();
        hasher.update(&buffer);
        let mut hash = hasher.finalize().as_slice().to_vec();
        hash.insert(0, 0);

        let exp = BigInt::parse_bytes(exponent, 10).unwrap_or_default();
        let mud = BigInt::parse_bytes(modulus, 10).unwrap_or_default();
        let rsa = BigInt::from_bytes_be(Sign::Plus, &hash)
            .modpow(&exp, &mud)
            .to_bytes_be()
            .1;

        buffer.extend(rsa);

        Ok(buffer)
    }
}

impl IntoIterator for Checksum {
    type Item = Entry;
    type IntoIter = std::vec::IntoIter<Entry>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl<'a> IntoIterator for &'a Checksum {
    type Item = &'a Entry;
    type IntoIter = Iter<'a, Entry>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}

impl<'a> IntoIterator for &'a mut Checksum {
    type Item = &'a mut Entry;
    type IntoIter = IterMut<'a, Entry>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter_mut()
    }
}

impl Default for Entry {
    #[inline]
    fn default() -> Self {
        Self {
            crc: 0,
            version: 0,
            #[cfg(feature = "rs3")]
            hash: vec![0; 64],
        }
    }
}
