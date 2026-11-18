use core::{
  borrow::{Borrow, BorrowMut},
  cmp::Ordering,
  hash::{Hash, Hasher},
};
use mayber::{Maybe, MaybeMut, MaybeRef};
use std::collections::hash_map::DefaultHasher;

fn hash(value: impl Hash) -> u64 {
  let mut state = DefaultHasher::new();
  value.hash(&mut state);
  state.finish()
}

#[test]
fn reference_and_owned_keys_compare_and_hash_like_their_payload() {
  let n = 4i32;
  let borrowed: MaybeRef<'_, i32> = MaybeRef::from(&n);
  let owned = MaybeRef::from(4);
  assert_eq!(borrowed, owned);
  assert_eq!(borrowed.cmp(&owned), Ordering::Equal);
  assert!(borrowed < MaybeRef::Owned(5));
  assert!(owned == &4 && borrowed < &5);
  for value in [borrowed, owned] {
    assert_eq!(AsRef::<i32>::as_ref(&value), &4);
    assert_eq!(Borrow::<i32>::borrow(&value), &4);
    assert_eq!(hash(value), hash(n));
    assert_eq!(format!("{value}"), "4");
  }
  assert_eq!(format!("{:?}", Maybe::<bool, i32>::Ref(true)), "true");
  assert_eq!(format!("{:?}", Maybe::<bool, i32>::Owned(4)), "4");
}

#[test]
fn mutable_traits_write_through_the_selected_representation() {
  let mut original = 0;
  for mut value in [MaybeMut::from(&mut original), MaybeMut::default()] {
    *AsMut::<i32>::as_mut(&mut value) = 3;
    *BorrowMut::<i32>::borrow_mut(&mut value) += 4;
    assert_eq!(Borrow::<i32>::borrow(&value), &7);
    assert_eq!(AsRef::<i32>::as_ref(&value), &7);
    assert_eq!(*value, 7);
  }
  assert_eq!(original, 7);
}

#[test]
fn identical_payload_types_can_be_inspected_then_moved_out() {
  for value in [
    Maybe::Ref(String::from("move")),
    Maybe::Owned(String::from("move")),
  ] {
    let ptr = value.as_inner().as_ptr();
    let inner = value.into_inner();
    assert_eq!(inner, "move");
    assert_eq!(inner.as_ptr(), ptr);
  }
}

#[test]
fn optional_views_preserve_variant_and_mutate_in_place() {
  for mut value in [
    Maybe::Ref(Some(1)),
    Maybe::Owned(Some(1)),
    Maybe::Ref(None),
    Maybe::Owned(None),
  ] {
    let owned = value.is_owned();
    assert_eq!(value.is_some(), !value.is_none());
    let view = value.as_option_mut();
    assert_eq!(view.is_owned(), owned);
    if let Some(inner) = view.into_inner() {
      *inner += 1;
    }
    let shared = value.as_option_ref();
    assert_eq!(shared.is_owned(), owned);
    assert_eq!(shared.into_inner().copied(), value.into_inner().map(|_| 2));
  }
}

#[test]
fn uninitialized_constructors_provide_writable_slots() {
  type Pair = Maybe<u8, u16>;
  let mut slot = Pair::uninit();
  assert_eq!(*slot.write(Pair::Owned(7)).as_maybe_ref().unwrap_owned(), 7);
  let mut array = Pair::uninit_array::<2>();
  for (index, slot) in array.iter_mut().enumerate() {
    assert_eq!(
      *slot
        .write(Pair::Ref(index as u8))
        .as_maybe_ref()
        .unwrap_ref(),
      index as u8
    );
  }
  assert_eq!(*Pair::uninit_ref().unwrap_ref().write(3), 3);
  assert_eq!(*Pair::uninit_owned().unwrap_owned().write(5), 5);
}

#[test]
fn mapping_and_recovering_a_variant_leaves_the_other_side_untouched() {
  type Pair = Maybe<u8, u16>;
  let left: Maybe<u16, u16> = Pair::Ref(3)
    .map(u16::from, |_| 0)
    .map_owned(|_| unreachable!());
  assert_eq!(left.try_unwrap_owned().unwrap_err().unwrap_ref(), 3);
  let right: Pair = Pair::Owned(8)
    .map_ref(|_| unreachable!())
    .map_owned(|v| v + 1);
  assert_eq!(right.try_unwrap_ref().unwrap_err().unwrap_owned(), 9);
  let moved = Pair::Ref(1).map(|v| v + 2, |v| v + 3);
  assert_eq!(moved.try_unwrap_ref().unwrap(), 3);
}

#[cfg(feature = "alloc")]
#[test]
fn borrowed_owned_views_keep_the_original_string_alive() {
  let source = String::from("hello");
  let owned: Maybe<&str, String> = Maybe::Owned(source);
  let clone = owned.to_owned();
  let view = owned.map_into_ref(|s| s.as_str());
  assert_eq!(view, clone);
  assert_ne!(view.as_ptr(), clone.as_ptr());
  assert_eq!(owned.into_owned(), clone);
}
