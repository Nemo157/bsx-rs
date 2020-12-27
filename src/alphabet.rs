//! Support for configurable alphabets

use core::fmt;

mod sealed {
    pub trait Sealed {}
}

/// A value that can be used as an alphabet for
/// [`EncodeBuilder::with_alphabet`](crate::encode::EncodeBuilder::with_alphabet) or
/// [`DecodeBuilder::with_alphabet`](crate::decode::DecodeBuilder::with_alphabet).
#[allow(clippy::len_without_is_empty)]
pub trait Alphabet: sealed::Sealed {
    /// The alphabet length.
    fn len(&self) -> usize;

    /// The mapping from numeric value to ASCII character while encoding.
    fn encode(&self) -> &[u8];

    /// The mapping from ASCII character to numeric value while decoding.
    fn decode(&self) -> &[u8];
}

/// Statically sized prepared Alphabet for
/// [`EncodeBuilder::with_alphabet`](crate::encode::EncodeBuilder::with_alphabet) and
/// [`DecodeBuilder::with_alphabet`](crate::decode::DecodeBuilder::with_alphabet).
#[derive(Clone, Copy)]
pub struct StaticAlphabet<const LEN: usize> {
    pub(crate) encode: [u8; LEN],
    pub(crate) decode: [u8; 128],
}

/// Dynamically sized prepared Alphabet for
/// [`EncodeBuilder::with_alphabet`](crate::encode::EncodeBuilder::with_alphabet) and
/// [`DecodeBuilder::with_alphabet`](crate::decode::DecodeBuilder::with_alphabet).
#[derive(Clone)]
pub struct DynamicAlphabet<A> {
    pub(crate) encode: A,
    pub(crate) decode: [u8; 128],
}

/// A placeholder for [`EncodeBuilder`](crate::encode::EncodeBuilder) and
/// [`DecodeBuilder`](crate::decode::DecodeBuilder) to indicate they have not yet been configured
/// with an alphabet.
#[derive(Copy, Clone, Debug)]
pub struct Unspecified;

/// Errors that could occur when preparing an [`StaticAlphabet`].
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

impl<const LEN: usize> StaticAlphabet<LEN> {
    /// Create prepared alphabet, checks that the alphabet is pure ASCII and that there are no
    /// duplicate characters, which would result in inconsistent encoding/decoding
    ///
    /// ```rust
    /// let symbolic = bsx::StaticAlphabet::new(
    ///     b" !\"#$%&'()*+,-./0123456789:;<=>?@"
    /// )?;
    ///
    /// let decoded = bsx::decode("he11owor1d")
    ///     .with_alphabet(bsx::StaticAlphabet::RIPPLE)
    ///     .into_vec()?;
    /// let encoded = bsx::encode(decoded)
    ///     .with_alphabet(&symbolic)
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
    ///     bsx::StaticAlphabet::new(b"aa").unwrap_err());
    /// ```
    ///
    /// ### Non-ASCII Character
    ///
    /// ```rust
    /// assert_eq!(
    ///     bsx::alphabet::Error::NonAsciiCharacter { index: 1 },
    ///     bsx::StaticAlphabet::new(&[b'a', 255]).unwrap_err());
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
    /// const SYMBOLIC: &'static bsx::StaticAlphabet<33> = &bsx::StaticAlphabet::new_unwrap(
    ///     b" !\"#$%&'()*+,-./0123456789:;<=>?@"
    /// );
    ///
    /// let decoded = bsx::decode("he11owor1d")
    ///     .with_alphabet(bsx::StaticAlphabet::RIPPLE)
    ///     .into_vec()?;
    /// let encoded = bsx::encode(decoded).with_alphabet(SYMBOLIC)
    ///     .into_string();
    ///
    /// assert_eq!("174<+/-1:0>", encoded);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// If your alphabet is inconsistent then this will fail to compile in a `const` context:
    ///
    /// ```compile_fail
    /// const _: &'static bsx::StaticAlphabet::<2> = &bsx::StaticAlphabet::new_unwrap(
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

impl<A: AsRef<[u8]>> DynamicAlphabet<A> {
    /// Create prepared alphabet, checks that the alphabet is pure ASCII and that there are no
    /// duplicate characters, which would result in inconsistent encoding/decoding
    ///
    /// ```rust
    /// let symbolic = bsx::DynamicAlphabet::new(
    ///     b" !\"#$%&'()*+,-./:;<=>?@"
    /// )?;
    ///
    /// let decoded = bsx::decode("he11owor1d")
    ///     .with_alphabet(bsx::StaticAlphabet::RIPPLE)
    ///     .into_vec()?;
    /// let encoded = bsx::encode(decoded)
    ///     .with_alphabet(&symbolic)
    ///     .into_string();
    ///
    /// assert_eq!(r#"!%*@-<!"?!"++"#, encoded);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    /// ## Errors
    ///
    /// ### Duplicate Character
    ///
    /// ```rust
    /// assert_eq!(
    ///     bsx::alphabet::Error::DuplicateCharacter { character: 'a', first: 0, second: 1 },
    ///     bsx::DynamicAlphabet::new("aa").unwrap_err());
    /// ```
    ///
    /// ### Non-ASCII Character
    ///
    /// ```rust
    /// assert_eq!(
    ///     bsx::alphabet::Error::NonAsciiCharacter { index: 1 },
    ///     bsx::DynamicAlphabet::new(&[b'a', 255]).unwrap_err());
    /// ```
    pub fn new(base: A) -> Result<Self, Error> {
        let encode = base;
        let mut decode = [0xFF; 128];

        for (i, &c) in encode.as_ref().iter().enumerate() {
            if c >= 128 {
                return Err(Error::NonAsciiCharacter { index: i });
            }
            if decode[c as usize] != 0xFF {
                return Err(Error::DuplicateCharacter {
                    character: c as char,
                    first: decode[c as usize] as usize,
                    second: i,
                });
            }
            decode[c as usize] = i as u8;
        }

        Ok(Self { encode, decode })
    }
}

impl dyn Alphabet {
    /// Bitcoin's alphabet as defined in their Base58Check encoding.
    ///
    /// See <https://en.bitcoin.it/wiki/Base58Check_encoding#Base58_symbol_chart>
    pub const BITCOIN: &'static Self = &StaticAlphabet::BITCOIN;

    /// Monero's alphabet as defined in this forum post.
    ///
    /// See <https://forum.getmonero.org/4/academic-and-technical/221/creating-a-standard-for-physical-coins>
    pub const MONERO: &'static Self = &StaticAlphabet::MONERO;

    /// Ripple's alphabet as defined in their wiki.
    ///
    /// See <https://wiki.ripple.com/Encodings>
    pub const RIPPLE: &'static Self = &StaticAlphabet::RIPPLE;

    /// Flickr's alphabet for creating short urls from photo ids.
    ///
    /// See <https://www.flickr.com/groups/api/discuss/72157616713786392/>
    pub const FLICKR: &'static Self = &StaticAlphabet::FLICKR;
}

impl StaticAlphabet<58> {
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

impl<const LEN: usize> sealed::Sealed for StaticAlphabet<LEN> {}

impl<const LEN: usize> Alphabet for StaticAlphabet<LEN> {
    fn len(&self) -> usize {
        LEN
    }

    fn encode(&self) -> &[u8] {
        &self.encode
    }

    fn decode(&self) -> &[u8] {
        &self.decode
    }
}

impl<const LEN: usize> fmt::Debug for StaticAlphabet<LEN> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(s) = core::str::from_utf8(&self.encode) {
            f.debug_tuple("StaticAlphabet").field(&s).finish()
        } else {
            unreachable!()
        }
    }
}

impl<A> sealed::Sealed for DynamicAlphabet<A> {}

impl<A: AsRef<[u8]>> Alphabet for DynamicAlphabet<A> {
    fn len(&self) -> usize {
        self.encode.as_ref().len()
    }

    fn encode(&self) -> &[u8] {
        self.encode.as_ref()
    }

    fn decode(&self) -> &[u8] {
        &self.decode
    }
}

impl<A: AsRef<[u8]>> fmt::Debug for DynamicAlphabet<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(s) = core::str::from_utf8(self.encode.as_ref()) {
            f.debug_tuple("DynamicAlphabet").field(&s).finish()
        } else {
            unreachable!()
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl<A: sealed::Sealed + ?Sized> sealed::Sealed for &A {}

impl<A: Alphabet + ?Sized> Alphabet for &A {
    fn len(&self) -> usize {
        (**self).len()
    }
    fn encode(&self) -> &[u8] {
        (**self).encode()
    }
    fn decode(&self) -> &[u8] {
        (**self).decode()
    }
}

// Force evaluation of the associated constants to make sure they don't error
const _: () = {
    let _ = StaticAlphabet::<58>::BITCOIN;
    let _ = StaticAlphabet::<58>::MONERO;
    let _ = StaticAlphabet::<58>::RIPPLE;
    let _ = StaticAlphabet::<58>::FLICKR;
};

#[test]
#[should_panic]
fn test_new_unwrap_does_panic() {
    StaticAlphabet::new_unwrap(b"aa");
}
