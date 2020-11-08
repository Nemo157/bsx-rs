mod cases;

#[cfg(feature = "check")]
use assert_matches::assert_matches;

#[test]
fn test_decode() {
    for &(val, s) in cases::TEST_CASES.iter() {
        assert_eq!(val.to_vec(), bsx::decode(s, bsx::Alphabet::<58>::BITCOIN).into_vec().unwrap());
    }
}

#[test]
fn test_decode_small_buffer_err() {
    let mut output = [0; 2];
    assert_eq!(
        bsx::decode("a3gV", bsx::Alphabet::<58>::BITCOIN).into(&mut output),
        Err(bsx::decode::Error::BufferTooSmall)
    );
}

#[test]
fn test_decode_invalid_char() {
    let sample = "123456789abcd!efghij";
    assert_eq!(
        bsx::decode(sample, bsx::Alphabet::<58>::BITCOIN).into_vec().unwrap_err(),
        bsx::decode::Error::InvalidCharacter {
            character: '!',
            index: 13
        }
    );
}
