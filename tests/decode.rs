mod cases;

#[cfg(feature = "check")]
use assert_matches::assert_matches;

#[test]
fn test_decode() {
    for &(val, s) in cases::TEST_CASES.iter() {
        assert_eq!(
            val.to_vec(),
            bsx::decode(s)
                .with_alphabet(bsx::StaticAlphabet::BITCOIN)
                .into_vec()
                .unwrap()
        );
    }
}

#[test]
fn test_decode_small_buffer_err() {
    let mut output = [0; 2];
    assert_eq!(
        bsx::decode("a3gV")
            .with_alphabet(bsx::StaticAlphabet::BITCOIN)
            .into(&mut output),
        Err(bsx::decode::Error::BufferTooSmall)
    );
}

#[test]
fn test_decode_invalid_char() {
    let sample = "123456789abcd!efghij";
    assert_eq!(
        bsx::decode(sample)
            .with_alphabet(bsx::StaticAlphabet::<58>::BITCOIN)
            .into_vec()
            .unwrap_err(),
        bsx::decode::Error::InvalidCharacter {
            character: '!',
            index: 13
        }
    );
}
