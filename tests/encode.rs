mod cases;

const FILLER: [u8; 512] = [b'~'; 512];

#[test]
fn test_encode() {
    for &(val, s) in cases::TEST_CASES.iter() {
        assert_eq!(
            s,
            bsx::encode(val, bsx::Alphabet::<58>::BITCOIN).into_string()
        );

        assert_eq!(
            s.as_bytes(),
            &*bsx::encode(val, bsx::Alphabet::<58>::BITCOIN).into_vec()
        );

        {
            let mut bytes = FILLER;
            assert_eq!(
                Ok(s.len()),
                bsx::encode(val, bsx::Alphabet::<58>::BITCOIN).into(&mut bytes[..])
            );
            assert_eq!(s.as_bytes(), &bytes[..s.len()]);
            assert_eq!(&FILLER[s.len()..], &bytes[s.len()..]);
        }

        {
            let mut bytes = FILLER;
            if !s.is_empty() {
                bytes[(s.len() - 1)..=s.len()].copy_from_slice("Ę".as_bytes());
            }
            let string = core::str::from_utf8_mut(&mut bytes[..]).unwrap();
            assert_eq!(
                Ok(s.len()),
                bsx::encode(val, bsx::Alphabet::<58>::BITCOIN).into(string)
            );
            assert_eq!(s.as_bytes(), &bytes[..s.len()]);
            if !s.is_empty() {
                assert_eq!(0, bytes[s.len()]);
            }
            assert_eq!(&FILLER[(s.len() + 1)..], &bytes[(s.len() + 1)..]);
        }
    }
}
