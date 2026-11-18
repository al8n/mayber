use crate::Maybe;
use core::borrow::Borrow;
use rkyv::{munge::munge, rancor::Fallible, Archive, Deserialize, Place, Portable, Serialize};

/// Archived contents of a `Maybe<R, T>` whose reference representation borrows `T`.
/// Both variants have the same archive format. Deserialization constructs `Owned`.
#[derive(Portable, rkyv::bytecheck::CheckBytes)]
#[bytecheck(crate = rkyv::bytecheck)]
#[repr(transparent)]
pub struct ArchivedMaybe<T: Portable>(T);

impl<T: Portable> ArchivedMaybe<T> {
  /// Borrows the archived payload without deserialization.
  pub fn as_inner(&self) -> &T {
    &self.0
  }
}

impl<R: Borrow<T>, T: Archive> Archive for Maybe<R, T> {
  type Archived = ArchivedMaybe<T::Archived>;
  type Resolver = T::Resolver;
  fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
    munge!(let ArchivedMaybe(inner) = out);
    self.borrow_as::<T>().resolve(resolver, inner);
  }
}

impl<R: Borrow<T>, T: Serialize<S>, S: Fallible + ?Sized> Serialize<S> for Maybe<R, T> {
  fn serialize(&self, s: &mut S) -> Result<Self::Resolver, S::Error> {
    self.borrow_as::<T>().serialize(s)
  }
}

impl<R, T: Archive, D: Fallible + ?Sized> Deserialize<Maybe<R, T>, D> for ArchivedMaybe<T::Archived>
where
  T::Archived: Deserialize<T, D>,
{
  fn deserialize(&self, d: &mut D) -> Result<Maybe<R, T>, D::Error> {
    self.0.deserialize(d).map(Maybe::Owned)
  }
}
