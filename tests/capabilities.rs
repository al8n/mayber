use core::{cell::Cell, mem::MaybeUninit};
use mayber::{Maybe, MaybeMut, MaybeRef};

#[test]
fn fold_moves_or_borrows_only_the_active_payload() {
  let text: Maybe<String, usize> = Maybe::Ref(String::from("hi"));
  let suffix = String::from("!");
  assert_eq!(text.fold(move |s| s + &suffix, |_| unreachable!()), "hi!");

  let mut numbers: Maybe<String, Vec<u8>> = Maybe::Owned(vec![3]);
  numbers
    .as_maybe_mut()
    .fold(|_| unreachable!(), |v| v.push(7));
  let view = numbers
    .as_maybe_ref()
    .fold(|s| s.as_bytes(), |v| v.as_slice());
  assert_eq!(view, &[3, 7]);
}

#[test]
fn either_payload_exposes_the_same_unsized_view() {
  let text = String::from("borrowed");
  let value: Maybe<&str, String> = Maybe::Ref(&text);
  assert!(core::ptr::eq(value.borrow_as::<str>(), text.as_str()));
  let bytes = vec![1, 2, 3];
  let ptr = bytes.as_ptr();
  let value: Maybe<&[u8], Vec<u8>> = Maybe::Owned(bytes);
  let slice: &[u8] = value.borrow_as();
  assert_eq!(slice, &[1, 2, 3]);
  assert_eq!(slice.as_ptr(), ptr);
}

#[test]
fn mutable_to_shared_preserves_the_borrow_or_moves_the_owner() {
  struct Item(u8);
  fn freeze(value: MaybeMut<'_, Item>) -> MaybeRef<'_, Item> {
    value.into()
  }
  let mut item = Item(7);
  let ptr = core::ptr::from_ref(&item);
  let mut value = MaybeMut::Ref(&mut item);
  value.as_mut().0 += 1;
  let shared = freeze(value).unwrap_ref();
  assert_eq!(shared.0, 8);
  assert_eq!(core::ptr::from_ref(shared), ptr);
  assert_eq!(freeze(MaybeMut::Owned(Item(9))).unwrap_owned().0, 9);

  const READS: (u8, u8) = {
    let mut n = 3;
    (
      *MaybeMut::Ref(&mut n).as_ref(),
      *MaybeMut::Owned(5).as_ref(),
    )
  };
  assert_eq!(READS, (3, 5));
}

#[test]
fn clone_from_delegates_for_matching_variants_and_replaces_otherwise() {
  type Pair = Maybe<Vec<u8>, Vec<u8>>;
  let mut buf = Vec::with_capacity(8);
  buf.push(0);
  let ptr = buf.as_ptr();
  let mut value = Pair::Ref(buf);
  value.clone_from(&Pair::Ref(vec![1, 2]));
  let buf = value.unwrap_ref();
  assert_eq!(buf, [1, 2]);
  assert_eq!(buf.as_ptr(), ptr);
  let mut value = Pair::Owned(buf);
  value.clone_from(&Pair::Owned(vec![3, 4]));
  let buf = value.unwrap_owned();
  assert_eq!(buf, [3, 4]);
  assert_eq!(buf.as_ptr(), ptr);
  let mut value = Pair::Owned(buf);
  value.clone_from(&Pair::Ref(vec![5]));
  assert!(matches!(&value, Maybe::Ref(v) if v == &[5]));
  value.clone_from(&Pair::Owned(vec![6]));
  assert_eq!(value.unwrap_owned(), [6]);
}

#[test]
fn generic_payload_pairs_are_copyable_or_cloneable() {
  let view: Maybe<(u8, u8), u32> = Maybe::Ref((3, 7));
  let copy = view;
  assert_eq!(copy.unwrap_ref(), view.unwrap_ref());

  let mut view: Maybe<String, Vec<u8>> = Maybe::Ref(String::from("abc"));
  let cloned = view.clone();
  if let Maybe::Ref(text) = &mut view {
    text.clear();
  }
  assert_eq!(cloned.unwrap_ref(), "abc");
  let owned: Maybe<String, Vec<u8>> = Maybe::Owned(vec![3, 7]);
  assert_eq!(owned.clone().unwrap_owned(), owned.unwrap_owned());
}

#[test]
fn owning_a_span_clones_only_the_borrowed_variant() {
  struct Span<'a>(&'a Cell<usize>);
  impl Clone for Span<'_> {
    fn clone(&self) -> Self {
      self.0.set(self.0.get() + 1);
      Self(self.0)
    }
  }

  let clones = Cell::new(0);
  let span = Span(&clones);
  let _moved = MaybeRef::Owned(span).map_into_owned(Clone::clone);
  assert_eq!(clones.get(), 0);
  let span = Span(&clones);
  let _cloned = MaybeRef::Ref(&span).map_into_owned(Clone::clone);
  assert_eq!(clones.get(), 1);
}

#[test]
fn mutable_borrow_writes_back_and_owned_payload_moves_out() {
  struct Counter(usize);
  let mut original = Counter(3);
  let mut borrowed = MaybeMut::Ref(&mut original);
  borrowed.0 += 1;
  assert_eq!(original.0, 4);

  let mut owned = MaybeMut::Owned(Counter(7));
  owned.0 += 1;
  assert_eq!(owned.unwrap_owned().0, 8);
}

#[test]
fn assume_init_preserves_the_variant_and_drops_each_payload_once() {
  struct Probe<'a>(&'a Cell<usize>);
  impl Drop for Probe<'_> {
    fn drop(&mut self) {
      self.0.set(self.0.get() + 1);
    }
  }

  for ref_variant in [true, false] {
    let drops = Cell::new(0);
    let mut slot: Maybe<MaybeUninit<Probe<'_>>, MaybeUninit<Probe<'_>>> = if ref_variant {
      Maybe::uninit_both_ref()
    } else {
      Maybe::uninit_both_owned()
    };
    match &mut slot {
      Maybe::Ref(payload) | Maybe::Owned(payload) => payload.write(Probe(&drops)),
    };
    // SAFETY: the active slot was initialized above and has not been read or dropped.
    let value = unsafe { slot.assume_init() };
    assert_eq!(value.is_ref(), ref_variant);
    assert_eq!(drops.get(), 0);
    drop(value);
    assert_eq!(drops.get(), 1);
  }
}

#[test]
fn into_owned_clones_only_borrowed_payloads_in_every_tier() {
  struct Span<'a>(&'a Cell<u32>);
  impl Clone for Span<'_> {
    fn clone(&self) -> Self {
      self.0.set(self.0.get() + 1);
      Self(self.0)
    }
  }

  let calls = Cell::new(0);
  let mut original = Span(&calls);
  let _ = MaybeRef::Ref(&original).into_owned();
  let _ = MaybeMut::Ref(&mut original).into_owned();
  assert_eq!(calls.get(), 2);
  let owned = MaybeRef::Owned(original).into_owned();
  let _ = MaybeMut::Owned(owned).into_owned();
  assert_eq!(calls.get(), 2);
}

#[test]
fn folds_lend_fields_of_heterogeneous_payloads() {
  type Entry = Maybe<(String, usize), String>;
  fn read(entry: &Entry) -> &str {
    entry.fold_ref(|tuple| tuple.0.as_str(), String::as_str)
  }
  fn edit(entry: &mut Entry) -> &mut String {
    entry.fold_mut(|tuple| &mut tuple.0, |s| s)
  }
  for mut entry in [
    Entry::Ref((String::from("a"), 1)),
    Entry::Owned(String::from("a")),
  ] {
    let was_ref = entry.is_ref();
    edit(&mut entry).push('b');
    assert_eq!(read(&entry), "ab");
    assert_eq!(entry.is_ref(), was_ref);
  }
}

#[test]
fn common_mutable_view_writes_through_and_preserves_variant() {
  let mut source = [1, 2];
  let mut borrowed: Maybe<&mut [u8], Vec<u8>> = Maybe::Ref(&mut source);
  borrowed.borrow_as_mut::<[u8]>()[0] = 7;
  assert!(borrowed.is_ref());
  assert_eq!(source, [7, 2]);
  let mut owned: Maybe<&mut [u8], Vec<u8>> = Maybe::Owned(vec![4, 5]);
  owned.borrow_as_mut::<[u8]>().reverse();
  assert_eq!(owned.unwrap_owned(), [5, 4]);
}

#[test]
fn failed_promotion_keeps_the_source_and_success_reuses_owned_storage() {
  type Block<'a> = Maybe<&'a [u8], [u8; 2]>;
  let source = [1, 2, 3];
  let mut block = Block::Ref(&source);
  assert!(block
    .try_make_owned_with(|r| <[u8; 2]>::try_from(*r))
    .is_err());
  assert!(core::ptr::eq(
    *block.as_maybe_ref().unwrap_ref(),
    &source[..]
  ));
  let slot = block.make_owned_with(|r| [r[0], r[1]]);
  slot[0] = 7;
  let again = block
    .try_make_owned_with::<()>(|_| panic!("already owned"))
    .unwrap();
  assert_eq!(again, &[7, 2]);
  block.make_owned_with(|_| panic!("already owned"))[1] = 8;
  assert_eq!(block.unwrap_owned(), [7, 8]);
  assert_eq!(source, [1, 2, 3]);
}

#[test]
fn transposing_moves_present_values_and_propagates_absence_or_errors() {
  type O = Maybe<Option<String>, Option<u16>>;
  assert_eq!(
    O::Ref(Some(String::from("a")))
      .transpose()
      .unwrap()
      .unwrap_ref(),
    "a"
  );
  assert_eq!(O::Owned(Some(3)).transpose().unwrap().unwrap_owned(), 3);
  assert!(O::Ref(None).transpose().is_none());
  assert!(O::Owned(None).transpose().is_none());

  type R = Maybe<Result<String, u8>, Result<u16, u8>>;
  assert_eq!(
    R::Ref(Ok(String::from("a")))
      .transpose()
      .unwrap()
      .unwrap_ref(),
    "a"
  );
  assert_eq!(R::Owned(Ok(3)).transpose().unwrap().unwrap_owned(), 3);
  assert_eq!(R::Ref(Err(1)).transpose().unwrap_err(), 1);
  assert_eq!(R::Owned(Err(2)).transpose().unwrap_err(), 2);
}

#[test]
fn fallible_maps_only_run_the_selected_conversion() {
  type Pair = Maybe<&'static str, u16>;
  let mapped: Maybe<u8, u8> = Pair::Ref("17")
    .try_map(str::parse, |_| unreachable!())
    .unwrap();
  assert_eq!(mapped.unwrap_ref(), 17);
  let error = Pair::Owned(300).try_map(|_| -> Result<(), _> { unreachable!() }, u8::try_from);
  assert!(error.is_err());
}

#[test]
fn equal_payloads_can_be_edited_in_const_code() {
  const fn update(mut value: Maybe<u8, u8>) -> Maybe<u8, u8> {
    *value.as_inner_mut() += 1;
    value
  }
  const LEFT: Maybe<u8, u8> = update(Maybe::Ref(2));
  const RIGHT: Maybe<u8, u8> = update(Maybe::Owned(2));
  assert!(matches!(LEFT, Maybe::Ref(3)));
  assert!(matches!(RIGHT, Maybe::Owned(3)));
}
