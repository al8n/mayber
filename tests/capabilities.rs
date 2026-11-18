use core::{cell::Cell, mem::MaybeUninit};
use mayber::{Maybe, MaybeMut, MaybeRef};

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
