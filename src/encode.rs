//! Functions for encoding arbitrary bases into strings.

use core::fmt;

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

use crate::Alphabet;

/// A builder for setting up the alphabet and output of an encode.
#[allow(missing_debug_implementations)]
pub struct EncodeBuilder<'a, I: AsRef<[u8]>, const LEN: usize> {
    input: I,
    alpha: &'a Alphabet<LEN>,
}

/// A specialized [`Result`](core::result::Result) type for [`bsx::encode`](module@crate::encode)
pub type Result<T> = core::result::Result<T, Error>;

/// Errors that could occur when encoding to a string.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// The output buffer was too small to contain the entire input.
    BufferTooSmall,
}

/// Represents a buffer that can be encoded into. See [`EncodeBuilder::into`] and the provided
/// implementations for more details.
pub trait EncodeTarget {
    /// Encodes into this buffer, provides the maximum length for implementations that wish to
    /// preallocate space, along with a function that will encode ASCII bytes into the buffer and
    /// return the length written to it.
    fn encode_with(
        &mut self,
        max_len: usize,
        f: impl for<'a> FnOnce(&'a mut [u8]) -> Result<usize>,
    ) -> Result<usize>;
}

impl<T: EncodeTarget + ?Sized> EncodeTarget for &mut T {
    fn encode_with(
        &mut self,
        max_len: usize,
        f: impl for<'a> FnOnce(&'a mut [u8]) -> Result<usize>,
    ) -> Result<usize> {
        T::encode_with(self, max_len, f)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
impl EncodeTarget for Vec<u8> {
    fn encode_with(
        &mut self,
        max_len: usize,
        f: impl for<'a> FnOnce(&'a mut [u8]) -> Result<usize>,
    ) -> Result<usize> {
        self.resize(max_len, 0);
        let len = f(&mut *self)?;
        self.truncate(len);
        Ok(len)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
impl EncodeTarget for String {
    fn encode_with(
        &mut self,
        max_len: usize,
        f: impl for<'a> FnOnce(&'a mut [u8]) -> Result<usize>,
    ) -> Result<usize> {
        let mut output = core::mem::replace(self, String::new()).into_bytes();
        let len = output.encode_with(max_len, f)?;
        *self = String::from_utf8(output).unwrap();
        Ok(len)
    }
}

impl EncodeTarget for [u8] {
    fn encode_with(
        &mut self,
        max_len: usize,
        f: impl for<'a> FnOnce(&'a mut [u8]) -> Result<usize>,
    ) -> Result<usize> {
        let _ = max_len;
        f(&mut *self)
    }
}

impl EncodeTarget for str {
    fn encode_with(
        &mut self,
        max_len: usize,
        f: impl for<'a> FnOnce(&'a mut [u8]) -> Result<usize>,
    ) -> Result<usize> {
        struct Guard<'a>(&'a mut [u8]);

        impl Drop for Guard<'_> {
            fn drop(&mut self) {
                let mut index = 0;
                loop {
                    match core::str::from_utf8(&self.0[index..]) {
                        Ok(_) => return,
                        Err(e) => {
                            index += e.valid_up_to();
                            if let Some(len) = e.error_len() {
                                for i in &mut self.0[index..index + len] {
                                    *i = 0;
                                }
                                index += len;
                            } else {
                                for i in &mut self.0[index..] {
                                    *i = 0;
                                }
                                index += self.0[index..].len();
                            }
                        }
                    }
                }
            }
        }

        let _ = max_len;

        let guard = Guard(unsafe { self.as_bytes_mut() });
        f(&mut *guard.0)
    }
}

impl<'a, I: AsRef<[u8]>, const LEN: usize> EncodeBuilder<'a, I, LEN> {
    const ENCODED_LEN_DIVISOR: usize = {
        if LEN.is_power_of_two() {
            LEN.trailing_zeros() as usize
        } else {
            (0usize.leading_zeros() - LEN.leading_zeros() - 1) as usize
        }
    };

    pub(crate) fn new(input: I, alpha: &'a Alphabet<LEN>) -> Self {
        EncodeBuilder { input, alpha }
    }

    /// Change the alphabet that will be used for encoding.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let input = [0x60, 0x65, 0xe7, 0x9b, 0xba, 0x2f, 0x78];
    /// assert_eq!(
    ///     "he11owor1d",
    ///     bsx::encode(input, bsx::Alphabet::<58>::BITCOIN)
    ///         .with_alphabet(bsx::Alphabet::RIPPLE)
    ///         .into_string());
    /// ```
    pub fn with_alphabet(self, alpha: &'a Alphabet<LEN>) -> Self {
        EncodeBuilder { alpha, ..self }
    }

    /// Encode into a new owned string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let input = [0x04, 0x30, 0x5e, 0x2b, 0x24, 0x73, 0xf0, 0x58];
    /// assert_eq!("he11owor1d", bsx::encode(input, bsx::Alphabet::<58>::BITCOIN).into_string());
    /// ```
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
    pub fn into_string(self) -> String {
        let mut output = String::new();
        self.into(&mut output).unwrap();
        output
    }

    /// Encode into a new owned vector.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let input = [0x04, 0x30, 0x5e, 0x2b, 0x24, 0x73, 0xf0, 0x58];
    /// assert_eq!(b"he11owor1d", &*bsx::encode(input, bsx::Alphabet::<58>::BITCOIN).into_vec());
    /// ```
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
    pub fn into_vec(self) -> Vec<u8> {
        let mut output = Vec::new();
        self.into(&mut output).unwrap();
        output
    }

    /// Encode into the given buffer.
    ///
    /// Returns the length written into the buffer.
    ///
    /// If the buffer is resizeable it will be reallocated to fit the encoded data and truncated to
    /// size.
    ///
    /// If the buffer is not resizeable bytes after the final character will be left alone, except
    /// up to 3 null bytes may be written to an `&mut str` to overwrite remaining characters of a
    /// partially overwritten multi-byte character.
    ///
    /// See the documentation for [`bsx::encode`](crate::encode()) for an
    /// explanation of the errors that may occur.
    ///
    /// # Examples
    ///
    /// ## `Vec<u8>`
    ///
    /// ```rust
    /// let input = [0x04, 0x30, 0x5e, 0x2b, 0x24, 0x73, 0xf0, 0x58];
    /// let mut output = "goodbye world".to_owned().into_bytes();
    /// bsx::encode(input, bsx::Alphabet::<58>::BITCOIN).into(&mut output)?;
    /// assert_eq!(b"he11owor1d", &*output);
    /// # Ok::<(), bsx::encode::Error>(())
    /// ```
    ///
    /// ## `&mut [u8]`
    ///
    /// ```rust
    /// let input = [0x04, 0x30, 0x5e, 0x2b, 0x24, 0x73, 0xf0, 0x58];
    /// let mut output = Vec::from("goodbye world");
    /// bsx::encode(input, bsx::Alphabet::<58>::BITCOIN).into(&mut output[..])?;
    /// assert_eq!(b"he11owor1drld", &*output);
    /// # Ok::<(), bsx::encode::Error>(())
    /// ```
    ///
    /// ## `String`
    ///
    /// ```rust
    /// let input = [0x04, 0x30, 0x5e, 0x2b, 0x24, 0x73, 0xf0, 0x58];
    /// let mut output = "goodbye world".to_owned();
    /// bsx::encode(input, bsx::Alphabet::<58>::BITCOIN).into(&mut output)?;
    /// assert_eq!("he11owor1d", output);
    /// # Ok::<(), bsx::encode::Error>(())
    /// ```
    ///
    /// ## `&mut str`
    ///
    /// ```rust
    /// let input = [0x04, 0x30, 0x5e, 0x2b, 0x24, 0x73, 0xf0, 0x58];
    /// let mut output = "goodbye world".to_owned();
    /// bsx::encode(input, bsx::Alphabet::<58>::BITCOIN).into(output.as_mut_str())?;
    /// assert_eq!("he11owor1drld", output);
    /// # Ok::<(), bsx::encode::Error>(())
    /// ```
    ///
    /// ### Clearing partially overwritten characters
    ///
    /// ```rust
    /// let input = [0x04, 0x30, 0x5e, 0x2b, 0x24, 0x73, 0xf0, 0x58];
    /// let mut output = "goodbye w®ld".to_owned();
    /// bsx::encode(input, bsx::Alphabet::<58>::BITCOIN).into(output.as_mut_str())?;
    /// assert_eq!("he11owor1d\0ld", output);
    /// # Ok::<(), bsx::encode::Error>(())
    /// ```
    pub fn into(self, mut output: impl EncodeTarget) -> Result<usize> {
        let max_encoded_len = (self.input.as_ref().len() * 8) / Self::ENCODED_LEN_DIVISOR + 1;
        output.encode_with(max_encoded_len, |output| {
            encode_into(self.input.as_ref(), output, &self.alpha)
        })
    }
}

fn encode_into<'a, I, const LEN: usize>(
    input: I,
    output: &mut [u8],
    alpha: &Alphabet<LEN>,
) -> Result<usize>
where
    I: Clone + IntoIterator<Item = &'a u8>,
{
    let mut index = 0;
    for &val in input.clone() {
        let mut carry = val as usize;
        for byte in &mut output[..index] {
            carry += (*byte as usize) << 8;
            *byte = (carry % LEN) as u8;
            carry /= LEN;
        }
        while carry > 0 {
            if index == output.len() {
                return Err(Error::BufferTooSmall);
            }
            output[index] = (carry % LEN) as u8;
            index += 1;
            carry /= LEN;
        }
    }

    for _ in input.into_iter().take_while(|v| **v == 0) {
        if index == output.len() {
            return Err(Error::BufferTooSmall);
        }
        output[index] = 0;
        index += 1;
    }

    for val in &mut output[..index] {
        *val = alpha.encode[*val as usize];
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
                write!(f, "buffer provided to encode string into was too small")
            }
        }
    }
}
