#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(variant_size_differences)]
#![doc(test(attr(deny(warnings))))]
#![feature(min_const_generics)]

//! Another arbitrary base codec implementation, using min-const-generics.
//!
//! # Features
//!
//!  Feature | Activation         | Effect
//! ---------|--------------------|--------
//!  `std`   | **on**-by-default  | Implement [`Error`](std::error::Error) for error types
//!  `alloc` | implied by `std`   | Support encoding/decoding to [`Vec`](alloc::vec::Vec) and [`String`](alloc::string::String) as appropriate
//!
//! # Examples
//!
//! ## Basic example
//!
//! ```rust
//! let decoded = bsx::decode("he11owor1d", bsx::Alphabet::<58>::BITCOIN).into_vec()?;
//! let encoded = bsx::encode(decoded, bsx::Alphabet::<58>::BITCOIN).into_string();
//! assert_eq!("he11owor1d", encoded);
//! # Ok::<(), bsx::decode::Error>(())
//! ```
//!
//! ## Changing the alphabet
//!
//! ```rust
//! let decoded = bsx::decode("he11owor1d", bsx::Alphabet::<58>::BITCOIN)
//!     .with_alphabet(bsx::Alphabet::RIPPLE)
//!     .into_vec()?;
//! let encoded = bsx::encode(decoded, bsx::Alphabet::<58>::BITCOIN)
//!     .with_alphabet(bsx::Alphabet::FLICKR)
//!     .into_string();
//! assert_eq!("4DSSNaN1SC", encoded);
//! # Ok::<(), bsx::decode::Error>(())
//! ```
//!
//! ## Decoding into an existing buffer
//!
//! ```rust
//! let (mut decoded, mut encoded) = ([0xFF; 8], String::with_capacity(10));
//! bsx::decode("he11owor1d", bsx::Alphabet::<58>::BITCOIN).into(&mut decoded)?;
//! bsx::encode(decoded, bsx::Alphabet::<58>::BITCOIN).into(&mut encoded)?;
//! assert_eq!("he11owor1d", encoded);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod alphabet;
#[doc(inline)]
pub use alphabet::Alphabet;

pub mod decode;
pub mod encode;

/// Setup decoder for the given string using the [default alphabet][Alphabet::DEFAULT].
///
/// # Examples
///
/// ## Basic example
///
/// ```rust
/// assert_eq!(
///     vec![0x04, 0x30, 0x5e, 0x2b, 0x24, 0x73, 0xf0, 0x58],
///     bsx::decode("he11owor1d", bsx::Alphabet::<58>::BITCOIN).into_vec()?);
/// # Ok::<(), bsx::decode::Error>(())
/// ```
///
/// ## Changing the alphabet
///
/// ```rust
/// assert_eq!(
///     vec![0x60, 0x65, 0xe7, 0x9b, 0xba, 0x2f, 0x78],
///     bsx::decode("he11owor1d", bsx::Alphabet::<58>::BITCOIN)
///         .with_alphabet(bsx::Alphabet::RIPPLE)
///         .into_vec()?);
/// # Ok::<(), bsx::decode::Error>(())
/// ```
///
/// ## Decoding into an existing buffer
///
/// ```rust
/// let mut output = [0xFF; 10];
/// assert_eq!(8, bsx::decode("he11owor1d", bsx::Alphabet::<58>::BITCOIN).into(&mut output)?);
/// assert_eq!(
///     [0x04, 0x30, 0x5e, 0x2b, 0x24, 0x73, 0xf0, 0x58, 0xFF, 0xFF],
///     output);
/// # Ok::<(), bsx::decode::Error>(())
/// ```
///
/// ## Errors
///
/// ### Invalid Character
///
/// ```rust
/// assert_eq!(
///     bsx::decode::Error::InvalidCharacter { character: 'l', index: 2 },
///     bsx::decode("hello world", bsx::Alphabet::<58>::BITCOIN).into_vec().unwrap_err());
/// ```
///
/// ### Non-ASCII Character
///
/// ```rust
/// assert_eq!(
///     bsx::decode::Error::NonAsciiCharacter { index: 5 },
///     bsx::decode("he11o🇳🇿", bsx::Alphabet::<58>::BITCOIN).into_vec().unwrap_err());
/// ```
///
/// ### Too Small Buffer
///
/// This error can only occur when reading into a provided buffer, when using
/// [`into_vec()`][decode::DecodeBuilder::into_vec] a vector large enough is guaranteed to be
/// used.
///
/// ```rust
/// let mut output = [0; 7];
/// assert_eq!(
///     bsx::decode::Error::BufferTooSmall,
///     bsx::decode("he11owor1d", bsx::Alphabet::<58>::BITCOIN).into(&mut output).unwrap_err());
/// ```
pub fn decode<'a, I: AsRef<[u8]>, const LEN: usize>(input: I, alphabet: &'a Alphabet<LEN>) -> decode::DecodeBuilder<'a, I, LEN> {
    decode::DecodeBuilder::new(input, alphabet)
}

/// Setup encoder for the given bytes using the [default alphabet][Alphabet::DEFAULT].
///
/// # Examples
///
/// ## Basic example
///
/// ```rust
/// let input = [0x04, 0x30, 0x5e, 0x2b, 0x24, 0x73, 0xf0, 0x58];
/// assert_eq!("he11owor1d", bsx::encode(input, bsx::Alphabet::<58>::BITCOIN).into_string());
/// ```
///
/// ## Changing the alphabet
///
/// ```rust
/// let input = [0x60, 0x65, 0xe7, 0x9b, 0xba, 0x2f, 0x78];
/// assert_eq!(
///     "he11owor1d",
///     bsx::encode(input, bsx::Alphabet::<58>::BITCOIN)
///         .with_alphabet(bsx::Alphabet::RIPPLE)
///         .into_string());
/// ```
///
/// ## Encoding into an existing string
///
/// ```rust
/// let input = [0x04, 0x30, 0x5e, 0x2b, 0x24, 0x73, 0xf0, 0x58];
/// let mut output = "goodbye world".to_owned();
/// bsx::encode(input, bsx::Alphabet::<58>::BITCOIN).into(&mut output)?;
/// assert_eq!("he11owor1d", output);
/// # Ok::<(), bsx::encode::Error>(())
/// ```
///
/// ## Errors
///
/// ### Too Small Buffer
///
/// This error can only occur when reading into an unresizeable buffer.
///
/// ```rust
/// let input = [0x04, 0x30, 0x5e, 0x2b, 0x24, 0x73, 0xf0, 0x58];
/// let mut output = [0; 7];
/// assert_eq!(
///     bsx::encode::Error::BufferTooSmall,
///     bsx::encode(input, bsx::Alphabet::<58>::BITCOIN).into(&mut output[..]).unwrap_err());
/// ```
pub fn encode<'a, I: AsRef<[u8]>, const LEN: usize>(input: I, alphabet: &'a Alphabet<LEN>) -> encode::EncodeBuilder<'a, I, LEN> {
    encode::EncodeBuilder::new(input, alphabet)
}
