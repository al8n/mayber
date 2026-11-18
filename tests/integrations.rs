#[cfg(feature = "arbitrary_1")]
#[test]
fn fuzz_bytes_select_the_variant_and_preserve_input_borrows() {
  use arbitrary_1::{Arbitrary, Unstructured};
  use mayber::Maybe;
  type Pair = Maybe<u8, u16>;
  assert!(matches!(
    Pair::arbitrary(&mut Unstructured::new(&[1, 7])),
    Ok(Maybe::Ref(7))
  ));
  assert!(matches!(
    Pair::arbitrary(&mut Unstructured::new(&[0, 1, 2])),
    Ok(Maybe::Owned(513))
  ));
  assert_eq!(Pair::size_hint(0), (2, Some(3)));
  assert_eq!(Pair::try_size_hint(0).unwrap(), Pair::size_hint(0));
  let bytes = [1, 42, 9];
  let borrowed = Maybe::<&[u8], ()>::arbitrary_take_rest(Unstructured::new(&bytes))
    .unwrap()
    .unwrap_ref();
  assert_eq!(borrowed, &bytes[1..]);
  assert_eq!(
    Pair::arbitrary_take_rest(Unstructured::new(&[0, 1, 2]))
      .unwrap()
      .unwrap_owned(),
    513
  );
  assert!(core::ptr::eq(borrowed.as_ptr(), bytes[1..].as_ptr()));
}

#[cfg(feature = "quickcheck_1")]
#[test]
fn quickcheck_generates_both_variants_and_shrinks_the_active_payload() {
  use mayber::Maybe;
  use quickcheck_1::{Arbitrary, Gen};
  let mut gen = Gen::from_size_and_seed(32, 42);
  let sample: Vec<_> = (0..64)
    .map(|_| Maybe::<u8, u16>::arbitrary(&mut gen))
    .collect();
  assert!(sample.iter().any(Maybe::is_owned) && sample.iter().any(Maybe::is_ref));
  for value in [Maybe::<u8, u16>::Ref(8), Maybe::Owned(16)] {
    let shrunk: Vec<_> = value.shrink().collect();
    assert!(!shrunk.is_empty());
    assert!(shrunk.iter().all(|s| s.is_ref() == value.is_ref()));
  }
}

#[cfg(feature = "proptest_1")]
#[test]
fn proptest_generates_both_payload_types_and_can_shrink() {
  use mayber::Maybe;
  use proptest_1::{
    arbitrary::any,
    strategy::{Strategy, ValueTree},
    test_runner::TestRunner,
  };
  let mut runner = TestRunner::deterministic();
  let strategy = any::<Maybe<u16, bool>>();
  let mut seen = (false, false, false);
  for _ in 0..64 {
    let mut tree = strategy.new_tree(&mut runner).unwrap();
    match tree.current() {
      Maybe::Ref(_) => seen.0 = true,
      Maybe::Owned(_) => seen.1 = true,
    }
    seen.2 |= tree.simplify();
  }
  assert_eq!(seen, (true, true, true));
}

#[cfg(feature = "borsh_1")]
#[test]
fn borsh_matches_payload_encoding_and_reads_owned_only() {
  use borsh_1::{from_slice, to_vec};
  use mayber::{Maybe, MaybeRef};
  let n = 0x1234u16;
  for v in [MaybeRef::Ref(&n), MaybeRef::Owned(n)] {
    assert_eq!(to_vec(&v).unwrap(), n.to_le_bytes());
  }
  struct NoCodec;
  assert_eq!(
    from_slice::<Maybe<NoCodec, u16>>(&n.to_le_bytes())
      .unwrap()
      .unwrap_owned(),
    n
  );
  assert!(from_slice::<Maybe<NoCodec, u16>>(&[1]).is_err());
}

#[cfg(feature = "schemars_1")]
#[test]
fn schema_matches_what_each_serde_contract_accepts() {
  use mayber::Maybe;
  use schemars_1::{generate::SchemaSettings, JsonSchema};
  type Pair = Maybe<bool, String>;
  let mut de = SchemaSettings::default().into_generator();
  assert_eq!(Pair::json_schema(&mut de), String::json_schema(&mut de));
  let mut ser = SchemaSettings::default().for_serialize().into_generator();
  let either = Pair::json_schema(&mut ser);
  let schemas = either.get("anyOf").unwrap().as_array().unwrap();
  assert_eq!(schemas.len(), 2);
  assert_eq!(&schemas[0], bool::json_schema(&mut ser).as_value());
  assert_eq!(&schemas[1], String::json_schema(&mut ser).as_value());
  assert_eq!(
    Maybe::<&str, String>::json_schema(&mut ser),
    String::json_schema(&mut ser)
  );
  assert_ne!(Pair::schema_name(), Maybe::<String, bool>::schema_name());
  assert_ne!(Pair::schema_id(), Maybe::<String, bool>::schema_id());
}

#[cfg(feature = "rkyv_0_8")]
#[test]
fn validated_archives_have_the_same_contents_and_decode_as_owned() {
  use core::mem::MaybeUninit;
  use mayber::{ArchivedMaybe, MaybeRef};
  use rkyv_0_8::{
    api::low::{access, from_bytes, to_bytes_in_with_alloc},
    rancor::Failure,
    ser::{allocator::SubAllocator, writer::Buffer},
    util::Align,
  };
  let n = 0x1234u32;
  for value in [MaybeRef::Ref(&n), MaybeRef::Owned(n)] {
    let mut output = Align([0u8; 64]);
    let mut scratch = [MaybeUninit::<u8>::uninit(); 64];
    let writer = to_bytes_in_with_alloc::<_, _, Failure>(
      &value,
      Buffer::from(&mut *output),
      SubAllocator::new(&mut scratch),
    )
    .unwrap();
    let bytes = &*writer;
    let archived = access::<ArchivedMaybe<rkyv_0_8::Archived<u32>>, Failure>(bytes).unwrap();
    assert_eq!(archived.as_inner().to_native(), n);
    assert_eq!(
      from_bytes::<MaybeRef<'_, u32>, Failure>(bytes)
        .unwrap()
        .unwrap_owned(),
      n
    );
    assert!(access::<ArchivedMaybe<rkyv_0_8::Archived<u32>>, Failure>(&bytes[..1]).is_err());
  }
}

#[cfg(feature = "zerocopy_0_8")]
#[test]
fn sized_pairs_have_known_layout_and_immutable_payloads() {
  fn check<T: zerocopy_0_8::Immutable + zerocopy_0_8::KnownLayout>() {}
  check::<mayber::Maybe<u16, u32>>();
  check::<mayber::MaybeRef<'_, u32>>();
}
