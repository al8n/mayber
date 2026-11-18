#![cfg(feature = "serde_1")]

use mayber::{Maybe, MaybeMut, MaybeRef};

#[test]
fn encodes_only_the_payload_and_decodes_as_owned() {
  let mut buf = [0; 8];
  let source = 7_u8;
  let mut mutable = source;
  assert_eq!(
    postcard::to_slice(&MaybeRef::Ref(&source), &mut buf).unwrap(),
    &[7]
  );
  assert_eq!(
    postcard::to_slice(&MaybeMut::Ref(&mut mutable), &mut buf).unwrap(),
    &[7]
  );
  let view: Maybe<(u8, u8), [u8; 2]> = Maybe::Ref((3, 7));
  let bytes = postcard::to_slice(&view, &mut buf).unwrap();
  assert_eq!(bytes, &[3, 7]);
  // Decoding never constructs R and does not require R: Deserialize.
  struct View;
  let decoded: Maybe<View, [u8; 2]> = postcard::from_bytes(bytes).unwrap();
  assert_eq!(decoded.unwrap_owned(), [3, 7]);
  let owned: Maybe<(u8, u8), [u8; 2]> = Maybe::Owned([3, 7]);
  assert_eq!(postcard::to_slice(&owned, &mut buf).unwrap(), &[3, 7]);
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[test]
fn borrowed_strings_and_slices_decode_to_owned_containers() {
  let mut buf = [0; 16];
  let text: Maybe<&str, String> = Maybe::Ref("hé");
  let bytes = postcard::to_slice(&text, &mut buf).unwrap();
  let decoded: Maybe<&str, String> = postcard::from_bytes(bytes).unwrap();
  assert_eq!(decoded.unwrap_owned(), "hé");

  let slice: Maybe<&[u8], Vec<u8>> = Maybe::Ref(&[0, 1, 255]);
  let bytes = postcard::to_slice(&slice, &mut buf).unwrap();
  let decoded: Maybe<&[u8], Vec<u8>> = postcard::from_bytes(bytes).unwrap();
  assert_eq!(decoded.unwrap_owned(), [0, 1, 255]);
}
