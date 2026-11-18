use crate::Maybe;

#[cfg(any(feature = "proptest_1", feature = "arbitrary_1"))]
fn from_result<R, T>(value: Result<R, T>) -> Maybe<R, T> {
  match value {
    Ok(value) => Maybe::Ref(value),
    Err(value) => Maybe::Owned(value),
  }
}

#[cfg(feature = "arbitrary_1")]
impl<'a, R: arbitrary_1::Arbitrary<'a>, T: arbitrary_1::Arbitrary<'a>> arbitrary_1::Arbitrary<'a>
  for Maybe<R, T>
{
  fn arbitrary(u: &mut arbitrary_1::Unstructured<'a>) -> arbitrary_1::Result<Self> {
    Result::<R, T>::arbitrary(u).map(from_result)
  }
  fn arbitrary_take_rest(mut u: arbitrary_1::Unstructured<'a>) -> arbitrary_1::Result<Self> {
    if bool::arbitrary(&mut u)? {
      R::arbitrary_take_rest(u).map(Maybe::Ref)
    } else {
      T::arbitrary_take_rest(u).map(Maybe::Owned)
    }
  }

  fn size_hint(depth: usize) -> (usize, Option<usize>) {
    Result::<R, T>::size_hint(depth)
  }
  fn try_size_hint(
    depth: usize,
  ) -> Result<(usize, Option<usize>), arbitrary_1::MaxRecursionReached> {
    Result::<R, T>::try_size_hint(depth)
  }
}

#[cfg(feature = "quickcheck_1")]
impl<R: quickcheck_1::Arbitrary, T: quickcheck_1::Arbitrary> quickcheck_1::Arbitrary
  for Maybe<R, T>
{
  fn arbitrary(g: &mut quickcheck_1::Gen) -> Self {
    if bool::arbitrary(g) {
      Self::Ref(R::arbitrary(g))
    } else {
      Self::Owned(T::arbitrary(g))
    }
  }
  fn shrink(&self) -> std::boxed::Box<dyn Iterator<Item = Self>> {
    match self {
      Self::Ref(r) => Box::new(r.shrink().map(Self::Ref)),
      Self::Owned(t) => Box::new(t.shrink().map(Self::Owned)),
    }
  }
}

#[cfg(feature = "proptest_1")]
impl<R: proptest_1::arbitrary::Arbitrary, T: proptest_1::arbitrary::Arbitrary>
  proptest_1::arbitrary::Arbitrary for Maybe<R, T>
{
  type Parameters = <Result<R, T> as proptest_1::arbitrary::Arbitrary>::Parameters;
  type Strategy = proptest_1::strategy::Map<
    <Result<R, T> as proptest_1::arbitrary::Arbitrary>::Strategy,
    fn(Result<R, T>) -> Self,
  >;
  fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
    use proptest_1::strategy::Strategy;
    Result::<R, T>::arbitrary_with(params).prop_map(from_result)
  }
}
