//! Support for configurable alphabets

use core::fmt;

/// Prepared Alphabet for
/// [`EncodeBuilder::with_alphabet`](crate::encode::EncodeBuilder::with_alphabet) and
/// [`DecodeBuilder::with_alphabet`](crate::decode::DecodeBuilder::with_alphabet).
#[derive(Clone, Copy)]
pub struct Alphabet<const LEN: usize> {
    pub(crate) encode: [u8; LEN],
    pub(crate) decode: [u8; 128],
}

/// Errors that could occur when preparing an [`Alphabet`].
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// The alphabet contained a duplicate character at at least 2 indexes.
    DuplicateCharacter {
        /// The duplicate character encountered.
        character: char,
        /// The first index the character was seen at.
        first: usize,
        /// The second index the character was seen at.
        second: usize,
    },

    /// The alphabet contained a multi-byte (or non-utf8) character.
    NonAsciiCharacter {
        /// The index at which the non-ASCII character was seen.
        index: usize,
    },
}

impl<const LEN: usize> Alphabet<LEN> {
    /// Create prepared alphabet, checks that the alphabet is pure ASCII and that there are no
    /// duplicate characters, which would result in inconsistent encoding/decoding
    ///
    /// ```rust
    /// let symbolic = bsx::Alphabet::new(
    ///     b" !\"#$%&'()*+,-./0123456789:;<=>?@"
    /// )?;
    ///
    /// let decoded = bsx::decode("he11owor1d", bsx::Alphabet::<58>::RIPPLE)
    ///     .into_vec()?;
    /// let encoded = bsx::encode(decoded, &symbolic)
    ///     .into_string();
    ///
    /// assert_eq!("174<+/-1:0>", encoded);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    /// ## Errors
    ///
    /// ### Duplicate Character
    ///
    /// ```rust
    /// assert_eq!(
    ///     bsx::alphabet::Error::DuplicateCharacter { character: 'a', first: 0, second: 1 },
    ///     bsx::Alphabet::new(b"aa").unwrap_err());
    /// ```
    ///
    /// ### Non-ASCII Character
    ///
    /// ```rust
    /// assert_eq!(
    ///     bsx::alphabet::Error::NonAsciiCharacter { index: 1 },
    ///     bsx::Alphabet::new(&[b'a', 255]).unwrap_err());
    /// ```
    pub const fn new(base: &[u8; LEN]) -> Result<Self, Error> {
        let mut encode = [0x00; LEN];
        let mut decode = [0xFF; 128];

        let mut i = 0;
        while i < encode.len() {
            if base[i] >= 128 {
                return Err(Error::NonAsciiCharacter { index: i });
            }
            if decode[base[i] as usize] != 0xFF {
                return Err(Error::DuplicateCharacter {
                    character: base[i] as char,
                    first: decode[base[i] as usize] as usize,
                    second: i,
                });
            }
            encode[i] = base[i];
            decode[base[i] as usize] = i as u8;
            i += 1;
        }

        Ok(Self { encode, decode })
    }

    /// Same as [`Self::new`], but gives a panic instead of an [`Err`] on bad input.
    ///
    /// Intended to support usage in `const` context until [`Result::unwrap`] is able to be called.
    ///
    /// ```rust
    /// const SYMBOLIC: &'static bsx::Alphabet<33> = &bsx::Alphabet::new_unwrap(
    ///     b" !\"#$%&'()*+,-./0123456789:;<=>?@"
    /// );
    ///
    /// let decoded = bsx::decode("he11owor1d", bsx::Alphabet::RIPPLE)
    ///     .into_vec()?;
    /// let encoded = bsx::encode(decoded, SYMBOLIC)
    ///     .into_string();
    ///
    /// assert_eq!("174<+/-1:0>", encoded);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// If your alphabet is inconsistent then this will fail to compile in a `const` context:
    ///
    /// ```compile_fail
    /// const _: &'static bsx::Alphabet::<2> = &bsx::Alphabet::new_unwrap(
    ///     b"aa"
    /// );
    /// ```
    pub const fn new_unwrap(base: &[u8; LEN]) -> Self {
        let result = Self::new(base);
        #[allow(unconditional_panic)] // https://github.com/rust-lang/rust/issues/78803
        [][match result {
            Ok(alphabet) => return alphabet,
            Err(_) => 0,
        }]
    }
}

impl Alphabet<58> {
    /// Bitcoin's alphabet as defined in their Base58Check encoding.
    ///
    /// See <https://en.bitcoin.it/wiki/Base58Check_encoding#Base58_symbol_chart>
    pub const BITCOIN: &'static Self =
        &Self::new_unwrap(b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz");

    /// Monero's alphabet as defined in this forum post.
    ///
    /// See <https://forum.getmonero.org/4/academic-and-technical/221/creating-a-standard-for-physical-coins>
    pub const MONERO: &'static Self =
        &Self::new_unwrap(b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz");

    /// Ripple's alphabet as defined in their wiki.
    ///
    /// See <https://wiki.ripple.com/Encodings>
    pub const RIPPLE: &'static Self =
        &Self::new_unwrap(b"rpshnaf39wBUDNEGHJKLM4PQRST7VWXYZ2bcdeCg65jkm8oFqi1tuvAxyz");

    /// Flickr's alphabet for creating short urls from photo ids.
    ///
    /// See <https://www.flickr.com/groups/api/discuss/72157616713786392/>
    pub const FLICKR: &'static Self =
        &Self::new_unwrap(b"123456789abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ");
}

impl<const LEN: usize> fmt::Debug for Alphabet<LEN> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(s) = core::str::from_utf8(&self.encode) {
            f.debug_tuple("Alphabet").field(&s).finish()
        } else {
            unreachable!()
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::DuplicateCharacter {
                character,
                first,
                second,
            } => write!(
                f,
                "alphabet contained a duplicate character `{}` at indexes {} and {}",
                character, first, second,
            ),
            Error::NonAsciiCharacter { index } => {
                write!(f, "alphabet contained a non-ascii character at {}", index)
            }
        }
    }
}

// Force evaluation of the associated constants to make sure they don't error
const _: () = {
    let _ = Alphabet::<58>::BITCOIN;
    let _ = Alphabet::<58>::MONERO;
    let _ = Alphabet::<58>::RIPPLE;
    let _ = Alphabet::<58>::FLICKR;
};

#[test]
#[should_panic]
fn test_new_unwrap_does_panic() {
    Alphabet::new_unwrap(b"aa");
}
