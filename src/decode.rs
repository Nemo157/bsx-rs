//! Functions for decoding arbitrary base encoded strings.

use core::fmt;

#[cfg(feature = "alloc")]
use alloc::{vec, vec::Vec};

use crate::{alphabet::Unspecified, Alphabet};

/// A builder for setting up the alphabet and output of a decode.
///
/// See the documentation for [`bsx::decode`](crate::decode()) for a more
/// high level view of how to use this.
#[allow(missing_debug_implementations)]
pub struct DecodeBuilder<I: AsRef<[u8]>, A> {
    input: I,
    alpha: A,
}

/// A specialized [`Result`](core::result::Result) type for [`bsx::decode`](module@crate::decode)
pub type Result<T> = core::result::Result<T, Error>;

/// Errors that could occur when decoding an arbitrary base encoded string.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// The output buffer was too small to contain the entire input.
    BufferTooSmall,

    /// The input contained a character that was not part of the current base's alphabet.
    InvalidCharacter {
        /// The unexpected character.
        character: char,
        /// The (byte) index in the input string the character was at.
        index: usize,
    },

    /// The input contained a multi-byte (or non-utf8) character which is
    /// unsupported by this decoder.
    NonAsciiCharacter {
        /// The (byte) index in the input string the start of the character was
        /// at.
        index: usize,
    },
}

impl<I: AsRef<[u8]>> DecodeBuilder<I, Unspecified> {
    pub(crate) fn new(input: I) -> Self {
        DecodeBuilder {
            input,
            alpha: Unspecified,
        }
    }
}

impl<I: AsRef<[u8]>, A> DecodeBuilder<I, A> {
    /// Change the alphabet that will be used for decoding.
    ///
    /// # Examples
    ///
    /// ```rust
    /// assert_eq!(
    ///     vec![0x60, 0x65, 0xe7, 0x9b, 0xba, 0x2f, 0x78],
    ///     bsx::decode("he11owor1d")
    ///         .with_alphabet(bsx::StaticAlphabet::RIPPLE)
    ///         .into_vec()?);
    /// # Ok::<(), bsx::decode::Error>(())
    /// ```
    pub fn with_alphabet<B>(self, alpha: B) -> DecodeBuilder<I, B> {
        DecodeBuilder {
            input: self.input,
            alpha,
        }
    }
}

impl<I: AsRef<[u8]>, A: Alphabet> DecodeBuilder<I, A> {
    /// Decode into a new vector of bytes.
    ///
    /// See the documentation for [`bsx::decode`](crate::decode()) for an
    /// explanation of the errors that may occur.
    ///
    /// # Examples
    ///
    /// ```rust
    /// assert_eq!(
    ///     vec![0x04, 0x30, 0x5e, 0x2b, 0x24, 0x73, 0xf0, 0x58],
    ///     bsx::decode("he11owor1d").with_alphabet(bsx::StaticAlphabet::BITCOIN).into_vec()?);
    /// # Ok::<(), bsx::decode::Error>(())
    /// ```
    ///
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
    pub fn into_vec(self) -> Result<Vec<u8>> {
        let mut output = vec![0; self.input.as_ref().len()];
        self.into(&mut output).map(|len| {
            output.truncate(len);
            output
        })
    }

    /// Decode into the given buffer.
    ///
    /// Returns the length written into the buffer, the rest of the bytes in
    /// the buffer will be untouched.
    ///
    /// See the documentation for [`bsx::decode`](crate::decode()) for an
    /// explanation of the errors that may occur.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut output = [0xFF; 10];
    /// assert_eq!(8, bsx::decode("he11owor1d").with_alphabet(bsx::StaticAlphabet::BITCOIN).into(&mut output)?);
    /// assert_eq!(
    ///     [0x04, 0x30, 0x5e, 0x2b, 0x24, 0x73, 0xf0, 0x58, 0xFF, 0xFF],
    ///     output);
    /// # Ok::<(), bsx::decode::Error>(())
    /// ```
    pub fn into<O: AsMut<[u8]>>(self, mut output: O) -> Result<usize> {
        decode_into(self.input.as_ref(), output.as_mut(), self.alpha)
    }
}

fn decode_into(input: &[u8], output: &mut [u8], alpha: impl Alphabet) -> Result<usize> {
    let mut index = 0;
    let (len, decode, encode) = (alpha.len(), alpha.decode(), alpha.encode());
    let zero = encode[0];

    for (i, c) in input.iter().enumerate() {
        if *c > 127 {
            return Err(Error::NonAsciiCharacter { index: i });
        }

        let mut val = decode[*c as usize] as usize;
        if val == 0xFF {
            return Err(Error::InvalidCharacter {
                character: *c as char,
                index: i,
            });
        }

        for byte in &mut output[..index] {
            val += (*byte as usize) * len;
            *byte = (val & 0xFF) as u8;
            val >>= 8;
        }

        while val > 0 {
            let byte = output.get_mut(index).ok_or(Error::BufferTooSmall)?;
            *byte = (val & 0xFF) as u8;
            index += 1;
            val >>= 8
        }
    }

    for _ in input.iter().take_while(|c| **c == zero) {
        let byte = output.get_mut(index).ok_or(Error::BufferTooSmall)?;
        *byte = 0;
        index += 1;
    }

    output[..index].reverse();
    Ok(index)
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::BufferTooSmall => {
                write!(f, "buffer provided to decode string into was too small")
            }
            Error::InvalidCharacter { character, index } => write!(
                f,
                "provided string contained invalid character {:?} at byte {}",
                character, index
            ),
            Error::NonAsciiCharacter { index } => write!(
                f,
                "provided string contained non-ascii character starting at byte {}",
                index
            ),
        }
    }
}
