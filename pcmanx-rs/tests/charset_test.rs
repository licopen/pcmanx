use pcmanx_rs::charset::{decode, encode, Encoding};

#[test]
fn utf8_decode_works() {
    let text = decode("hello".as_bytes(), Encoding::Utf8);
    assert_eq!(text, "hello");
}

#[test]
fn big5_decode_known_char() {
    let text = decode(&[0xA4, 0xA4], Encoding::Big5);
    assert_eq!(text, "中");
}

#[test]
fn utf8_encode_works() {
    let bytes = encode("hello", Encoding::Utf8);
    assert_eq!(bytes, b"hello");
}

#[test]
fn cjk_round_trip() {
    let src = "中文測試";
    let encoded = encode(src, Encoding::Big5);
    let decoded = decode(&encoded, Encoding::Big5);
    assert_eq!(decoded, src);
}
