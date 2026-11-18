use mayber::Maybe;
use std::{
  panic,
  sync::atomic::{AtomicUsize, Ordering},
};

// A separate integration-test binary keeps this process-wide hook isolated.
#[test]
fn unwrap_and_expect_report_the_callers_file() {
  static CALLER_LOCATIONS: AtomicUsize = AtomicUsize::new(0);
  let previous = panic::take_hook();
  panic::set_hook(Box::new(|info| {
    if info.location().is_some_and(|loc| loc.file() == file!()) {
      CALLER_LOCATIONS.fetch_add(1, Ordering::Relaxed);
    }
  }));
  type Pair = Maybe<u8, u8>;
  let operations: [fn() -> u8; 4] = [
    || Pair::Owned(1).unwrap_ref(),
    || Pair::Ref(1).unwrap_owned(),
    || Pair::Owned(1).expect_ref("expected Ref"),
    || Pair::Ref(1).expect_owned("expected Owned"),
  ];
  let panics = operations.map(|op| panic::catch_unwind(op).is_err());
  panic::set_hook(previous);
  assert_eq!(panics, [true; 4]);
  assert_eq!(CALLER_LOCATIONS.load(Ordering::Relaxed), 4);
}
