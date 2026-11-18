#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![deny(missing_docs)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc as std;

#[cfg(feature = "std")]
extern crate std;

use core::{
  borrow::{Borrow, BorrowMut},
  fmt,
  mem::MaybeUninit,
  ops::{Deref, DerefMut},
};

pub use crate::Maybe::{Owned, Ref};

/// A type alias for a [`Maybe`] enum that holds either a reference or an owned value.
///
/// This is the most common form of `Maybe`, representing either `&T` or `T`.
///
/// # Example
///
/// ```
/// use mayber::MaybeRef;
///
/// fn accepts_either(s: MaybeRef<String>) {
///     println!("Length: {}", s.len());
/// }
///
/// let owned = String::from("owned");
/// let borrowed = String::from("borrowed");
///
/// accepts_either(MaybeRef::Owned(owned));
/// accepts_either(MaybeRef::Ref(&borrowed));
/// ```
pub type MaybeRef<'a, T> = Maybe<&'a T, T>;

/// A type alias for a [`Maybe`] enum that holds either a mutable reference or an owned value.
///
/// This represents either `&mut T` or `T`.
///
/// # Example
///
/// ```
/// use mayber::MaybeMut;
///
/// fn modify(mut s: MaybeMut<String>) {
///     s.push_str(" world");
/// }
///
/// let mut owned = String::from("hello");
/// modify(MaybeMut::Owned(owned));
/// ```
///
/// Moving a mutable borrow preserves its exclusivity:
///
/// ```compile_fail
/// use mayber::MaybeMut;
/// let mut value = 0;
/// let borrowed = MaybeMut::Ref(&mut value);
/// let _ = Clone::clone(&borrowed);
/// ```
pub type MaybeMut<'a, T> = Maybe<&'a mut T, T>;

/// An enum that can hold either a reference or an owned value.
///
/// `Maybe<R, T>` is a generic enum that can contain either a reference type `R` or an owned type `T`.
/// The most common use cases are:
/// - [`MaybeRef<'a, T>`]: Either `&'a T` or `T`
/// - [`MaybeMut<'a, T>`]: Either `&'a mut T` or `T`
///
/// This type is particularly useful when you want to avoid unnecessary allocations or clones,
/// but still need the flexibility to work with owned data when necessary.
///
/// # Example
///
/// ```
/// use mayber::Maybe;
///
/// let ref_variant: Maybe<&str, String> = Maybe::Ref("hello");
/// let owned_variant: Maybe<&str, String> = Maybe::Owned("world".to_string());
///
/// assert!(ref_variant.is_ref());
/// assert!(owned_variant.is_owned());
/// ```
#[cfg_attr(
  feature = "zerocopy_0_8",
  derive(zerocopy_0_8::KnownLayout, zerocopy_0_8::Immutable)
)]
#[cfg_attr(feature = "zerocopy_0_8", zerocopy(crate = "zerocopy_0_8"))]
#[derive(Copy)]
pub enum Maybe<R, T> {
  /// A reference to a value
  Ref(R),
  /// An owned value
  Owned(T),
}

impl<R: Clone, T: Clone> Clone for Maybe<R, T> {
  #[inline]
  fn clone(&self) -> Self {
    match self {
      Self::Ref(value) => Self::Ref(value.clone()),
      Self::Owned(value) => Self::Owned(value.clone()),
    }
  }

  #[inline]
  fn clone_from(&mut self, source: &Self) {
    match (self, source) {
      (Self::Ref(target), Self::Ref(value)) => target.clone_from(value),
      (Self::Owned(target), Self::Owned(value)) => target.clone_from(value),
      (target, source) => *target = source.clone(),
    }
  }
}

impl<T> Borrow<T> for MaybeRef<'_, T> {
  #[inline]
  fn borrow(&self) -> &T {
    MaybeRef::as_ref(self)
  }
}

impl<T> Borrow<T> for MaybeMut<'_, T> {
  #[inline]
  fn borrow(&self) -> &T {
    MaybeMut::as_ref(self)
  }
}

impl<T> BorrowMut<T> for MaybeMut<'_, T> {
  #[inline]
  fn borrow_mut(&mut self) -> &mut T {
    MaybeMut::as_mut(self)
  }
}

impl<T> AsRef<T> for MaybeRef<'_, T> {
  #[inline]
  fn as_ref(&self) -> &T {
    MaybeRef::as_ref(self)
  }
}

impl<T> AsRef<T> for MaybeMut<'_, T> {
  #[inline]
  fn as_ref(&self) -> &T {
    MaybeMut::as_ref(self)
  }
}

impl<T> AsMut<T> for MaybeMut<'_, T> {
  #[inline]
  fn as_mut(&mut self) -> &mut T {
    MaybeMut::as_mut(self)
  }
}

impl<T> Deref for MaybeRef<'_, T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &Self::Target {
    MaybeRef::as_ref(self)
  }
}

impl<T> Deref for MaybeMut<'_, T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &Self::Target {
    MaybeMut::as_ref(self)
  }
}

impl<T> DerefMut for MaybeMut<'_, T> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    MaybeMut::as_mut(self)
  }
}

impl<T> PartialEq for MaybeRef<'_, T>
where
  T: PartialEq,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    (**self).eq(&**other)
  }
}

impl<T, O> PartialEq<&O> for MaybeRef<'_, T>
where
  T: PartialEq<O>,
{
  #[inline]
  fn eq(&self, other: &&O) -> bool {
    (**self).eq(*other)
  }
}

impl<T> Eq for MaybeRef<'_, T> where T: Eq {}

impl<T> PartialOrd for MaybeRef<'_, T>
where
  T: PartialOrd,
{
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
    (**self).partial_cmp(&**other)
  }
}

impl<T, O> PartialOrd<&O> for MaybeRef<'_, T>
where
  T: PartialOrd<O>,
{
  #[inline]
  fn partial_cmp(&self, other: &&O) -> Option<core::cmp::Ordering> {
    (**self).partial_cmp(*other)
  }
}

impl<T> Ord for MaybeRef<'_, T>
where
  T: Ord,
{
  #[inline]
  fn cmp(&self, other: &Self) -> core::cmp::Ordering {
    (**self).cmp(&**other)
  }
}

impl<T> fmt::Display for MaybeRef<'_, T>
where
  T: fmt::Display,
{
  #[inline]
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    (**self).fmt(f)
  }
}

impl<R: fmt::Debug, T: fmt::Debug> fmt::Debug for Maybe<R, T> {
  #[inline]
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Ref(r) => r.fmt(f),
      Self::Owned(t) => t.fmt(f),
    }
  }
}

impl<T> core::hash::Hash for MaybeRef<'_, T>
where
  T: core::hash::Hash,
{
  #[inline]
  fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
    (**self).hash(state)
  }
}

impl<R, T> Default for Maybe<R, T>
where
  T: Default,
{
  #[inline]
  fn default() -> Self {
    Maybe::Owned(T::default())
  }
}

impl<R, T> From<T> for Maybe<R, T> {
  #[inline]
  fn from(value: T) -> Self {
    Maybe::Owned(value)
  }
}

impl<'a, T> From<&'a T> for Maybe<&'a T, T> {
  #[inline]
  fn from(value: &'a T) -> Self {
    Maybe::Ref(value)
  }
}

impl<'a, T> From<&'a mut T> for Maybe<&'a mut T, T> {
  #[inline]
  fn from(value: &'a mut T) -> Self {
    Maybe::Ref(value)
  }
}

/// Preserves the original borrow lifetime or moves out the owned payload.
impl<'a, T> From<MaybeMut<'a, T>> for MaybeRef<'a, T> {
  #[inline]
  fn from(value: MaybeMut<'a, T>) -> Self {
    match value {
      Maybe::Ref(value) => Self::Ref(value),
      Maybe::Owned(value) => Self::Owned(value),
    }
  }
}

impl<T> Maybe<&T, T> {
  /// Converts the `Maybe<&T, T>` into a `&T`.
  #[inline]
  pub const fn as_ref(&self) -> &T {
    match self {
      Maybe::Ref(r) => r,
      Maybe::Owned(t) => t,
    }
  }
}

impl<T> Maybe<&mut T, T> {
  /// Borrows the contents for the lifetime of this shared borrow.
  #[inline]
  pub const fn as_ref(&self) -> &T {
    match self {
      Self::Ref(value) => value,
      Self::Owned(value) => value,
    }
  }

  /// Converts the `Maybe<&mut T, T>` into a `&mut T`.
  #[inline]
  pub const fn as_mut(&mut self) -> &mut T {
    match self {
      Maybe::Ref(r) => r,
      Maybe::Owned(t) => t,
    }
  }
}

impl<T> Maybe<T, T> {
  /// Converts the `Maybe<T, T>` into a `T`, consuming the `Maybe`.
  #[inline]
  pub fn into_inner(self) -> T {
    match self {
      Maybe::Ref(r) => r,
      Maybe::Owned(t) => t,
    }
  }

  /// Converts the `Maybe<T, T>` into a `&T`.
  #[inline]
  pub const fn as_inner(&self) -> &T {
    match self {
      Maybe::Ref(r) => r,
      Maybe::Owned(t) => t,
    }
  }
}

impl<R, T> Maybe<R, T> {
  /// Returns `true` if the Maybe contains a reference
  #[inline]
  pub const fn is_ref(&self) -> bool {
    matches!(self, Maybe::Ref(_))
  }

  /// Returns `true` if the Maybe contains an owned value
  #[inline]
  pub const fn is_owned(&self) -> bool {
    matches!(self, Maybe::Owned(_))
  }

  /// Applies the function for the active variant and returns its result.
  ///
  /// Use [`Self::as_maybe_ref`] or [`Self::as_maybe_mut`] first to fold borrowed payloads.
  ///
  /// ```
  /// use mayber::Maybe;
  ///
  /// let text: Maybe<&str, String> = Maybe::Ref("hello");
  /// assert_eq!(text.as_maybe_ref().fold(|s| s.len(), String::len), 5);
  /// ```
  #[inline]
  pub fn fold<U>(self, on_ref: impl FnOnce(R) -> U, on_owned: impl FnOnce(T) -> U) -> U {
    match self {
      Self::Ref(value) => on_ref(value),
      Self::Owned(value) => on_owned(value),
    }
  }

  /// Borrows the active payload as a type shared by both representations.
  ///
  /// ```
  /// use mayber::Maybe;
  ///
  /// let text: Maybe<&str, String> = Maybe::Owned(String::from("hello"));
  /// assert_eq!(text.borrow_as::<str>(), "hello");
  /// ```
  #[inline]
  pub fn borrow_as<U: ?Sized>(&self) -> &U
  where
    R: Borrow<U>,
    T: Borrow<U>,
  {
    match self {
      Self::Ref(value) => value.borrow(),
      Self::Owned(value) => value.borrow(),
    }
  }

  /// Maps a `Maybe<R, T>` to `Maybe<R2, T2>` by applying a function to the contained value.
  ///
  /// # Example
  ///
  /// ```
  /// use mayber::Maybe;
  ///
  /// let x: Maybe<&str, String> = Maybe::Owned("hello".to_string());
  /// let y = x.map(|s| s.len(), |s| s.len());
  /// match y {
  ///     Maybe::Owned(len) => assert_eq!(len, 5),
  ///     _ => panic!("Should be owned"),
  /// }
  /// ```
  #[inline]
  pub fn map<R2, T2, F, G>(self, ref_fn: F, owned_fn: G) -> Maybe<R2, T2>
  where
    F: FnOnce(R) -> R2,
    G: FnOnce(T) -> T2,
  {
    match self {
      Maybe::Ref(r) => Maybe::Ref(ref_fn(r)),
      Maybe::Owned(t) => Maybe::Owned(owned_fn(t)),
    }
  }

  /// Maps the reference variant by applying a function to it.
  ///
  /// # Example
  ///
  /// ```
  /// use mayber::Maybe;
  ///
  /// let x: Maybe<&str, String> = Maybe::Ref("hello");
  /// let y = x.map_ref(|s| s.len());
  /// match y {
  ///     Maybe::Ref(len) => assert_eq!(len, 5),
  ///     _ => panic!("Should be ref"),
  /// }
  /// ```
  #[inline]
  pub fn map_ref<R2, F>(self, f: F) -> Maybe<R2, T>
  where
    F: FnOnce(R) -> R2,
  {
    match self {
      Maybe::Ref(r) => Maybe::Ref(f(r)),
      Maybe::Owned(t) => Maybe::Owned(t),
    }
  }

  /// Maps the owned variant by applying a function to it.
  ///
  /// # Example
  ///
  /// ```
  /// use mayber::Maybe;
  ///
  /// let x: Maybe<&str, String> = Maybe::Owned("hello".to_string());
  /// let y = x.map_owned(|s| s.len());
  /// match y {
  ///     Maybe::Owned(len) => assert_eq!(len, 5),
  ///     _ => panic!("Should be owned"),
  /// }
  /// ```
  #[inline]
  pub fn map_owned<T2, F>(self, f: F) -> Maybe<R, T2>
  where
    F: FnOnce(T) -> T2,
  {
    match self {
      Maybe::Ref(r) => Maybe::Ref(r),
      Maybe::Owned(t) => Maybe::Owned(f(t)),
    }
  }

  /// Calls the provided closure with the contained value.
  ///
  /// # Example
  ///
  /// ```
  /// use mayber::Maybe;
  ///
  /// let x: Maybe<&str, String> = Maybe::Ref("hello");
  /// let result = x.inspect(
  ///     |s| assert_eq!(*s, "hello"),
  ///     |s| panic!("Should not be called")
  /// );
  /// assert!(result.is_ref());
  ///
  /// let y: Maybe<&str, String> = Maybe::Owned("world".to_string());
  /// let result = y.inspect(
  ///     |_| panic!("Should not be called"),
  ///     |s| assert_eq!(s, "world")
  /// );
  /// assert!(result.is_owned());
  /// ```
  #[inline]
  pub fn inspect<F, G>(self, ref_fn: F, owned_fn: G) -> Self
  where
    F: FnOnce(&R),
    G: FnOnce(&T),
  {
    match &self {
      Maybe::Ref(r) => ref_fn(r),
      Maybe::Owned(t) => owned_fn(t),
    }
    self
  }

  /// Unwraps the Maybe, yielding the reference if it is a `Ref` variant,
  /// otherwise panics.
  ///
  /// # Panics
  /// Panics if the Maybe is an `Owned` variant.
  ///
  /// # Example
  ///
  /// ```
  /// use mayber::Maybe;
  ///
  /// let x: Maybe<&str, String> = Maybe::Ref("hello");
  /// assert_eq!(x.unwrap_ref(), "hello");
  /// ```
  ///
  /// ```should_panic
  /// use mayber::Maybe;
  ///
  /// let x: Maybe<&str, String> = Maybe::Owned("hello".to_string());
  /// x.unwrap_ref(); // This will panic
  /// ```
  #[inline]
  #[track_caller]
  pub fn unwrap_ref(self) -> R {
    match self {
      Maybe::Ref(r) => r,
      Maybe::Owned(_) => panic!("called `Maybe::unwrap_ref()` on an `Owned` value"),
    }
  }

  /// Unwraps the Maybe, yielding the owned value if it is an `Owned` variant,
  /// otherwise panics.
  ///
  /// # Panics
  /// Panics if the Maybe is a `Ref` variant.
  ///
  /// # Example
  ///
  /// ```
  /// use mayber::Maybe;
  ///
  /// let x: Maybe<&str, String> = Maybe::Owned("hello".to_string());
  /// assert_eq!(x.unwrap_owned(), "hello");
  /// ```
  ///
  /// ```should_panic
  /// use mayber::Maybe;
  ///
  /// let x: Maybe<&str, String> = Maybe::Ref("hello");
  /// x.unwrap_owned(); // This will panic
  /// ```
  #[inline]
  #[track_caller]
  pub fn unwrap_owned(self) -> T {
    match self {
      Maybe::Ref(_) => panic!("called `Maybe::unwrap_owned()` on a `Ref` value"),
      Maybe::Owned(t) => t,
    }
  }

  /// Attempts to unwrap the Maybe, yielding the reference if it is a `Ref` variant,
  /// otherwise returns `Err(Self)`.
  ///
  /// # Example
  ///
  /// ```
  /// use mayber::Maybe;
  ///
  /// let x: Maybe<&str, String> = Maybe::Ref("hello");
  /// match x.try_unwrap_ref() {
  ///     Ok(s) => assert_eq!(s, "hello"),
  ///     Err(_) => panic!("Should be Ok"),
  /// }
  ///
  /// let y: Maybe<&str, String> = Maybe::Owned("world".to_string());
  /// assert!(y.try_unwrap_ref().is_err());
  /// ```
  #[inline]
  pub fn try_unwrap_ref(self) -> Result<R, Self> {
    match self {
      Maybe::Ref(r) => Ok(r),
      Maybe::Owned(_) => Err(self),
    }
  }

  /// Attempts to unwrap the Maybe, yielding the owned value if it is an `Owned` variant,
  /// otherwise returns `Err(Self)`.
  ///
  /// # Example
  ///
  /// ```
  /// use mayber::Maybe;
  ///
  /// let x: Maybe<&str, String> = Maybe::Owned("hello".to_string());
  /// match x.try_unwrap_owned() {
  ///     Ok(s) => assert_eq!(s, "hello"),
  ///     Err(_) => panic!("Should be Ok"),
  /// }
  ///
  /// let y: Maybe<&str, String> = Maybe::Ref("world");
  /// assert!(y.try_unwrap_owned().is_err());
  /// ```
  #[inline]
  pub fn try_unwrap_owned(self) -> Result<T, Self> {
    match self {
      Maybe::Ref(_) => Err(self),
      Maybe::Owned(t) => Ok(t),
    }
  }

  /// Unwraps the Maybe, yielding the reference if it is a `Ref` variant,
  ///
  /// # Panics
  /// Panics with a custom message if the Maybe is an `Owned` variant.
  ///
  /// # Example
  ///
  /// ```
  /// use mayber::Maybe;
  ///
  /// let x: Maybe<&str, String> = Maybe::Ref("hello");
  /// assert_eq!(x.expect_ref("should be ref"), "hello");
  /// ```
  ///
  /// ```should_panic
  /// use mayber::Maybe;
  ///
  /// let x: Maybe<&str, String> = Maybe::Owned("hello".to_string());
  /// x.expect_ref("Expected a reference"); // Panics with custom message
  /// ```
  #[inline]
  #[track_caller]
  pub fn expect_ref(self, msg: &str) -> R {
    match self {
      Maybe::Ref(r) => r,
      Maybe::Owned(_) => panic!("{msg}"),
    }
  }

  /// Unwraps the Maybe, yielding the owned value if it is an `Owned` variant,
  ///
  /// # Panics
  /// Panics with a custom message if the Maybe is a `Ref` variant.
  ///
  /// # Example
  ///
  /// ```
  /// use mayber::Maybe;
  ///
  /// let x: Maybe<&str, String> = Maybe::Owned("hello".to_string());
  /// assert_eq!(x.expect_owned("should be owned"), "hello");
  /// ```
  ///
  /// ```should_panic
  /// use mayber::Maybe;
  ///
  /// let x: Maybe<&str, String> = Maybe::Ref("hello");
  /// x.expect_owned("Expected an owned value"); // Panics with custom message
  /// ```
  #[inline]
  #[track_caller]
  pub fn expect_owned(self, msg: &str) -> T {
    match self {
      Maybe::Ref(_) => panic!("{msg}"),
      Maybe::Owned(t) => t,
    }
  }

  /// Converts the `Maybe<R, T>` into a `Maybe<&R, &T>`.
  ///
  /// # Example
  ///
  /// ```
  /// use mayber::Maybe;
  ///
  /// let x: Maybe<&str, String> = Maybe::Owned("hello".to_string());
  /// let y = x.as_maybe_ref();
  /// assert!(matches!(y, Maybe::Owned(&_)));
  ///
  /// let a: Maybe<&str, String> = Maybe::Ref("world");
  /// let b = a.as_maybe_ref();
  /// assert!(matches!(b, Maybe::Ref(&_)));
  /// ```
  #[inline]
  pub const fn as_maybe_ref(&self) -> Maybe<&R, &T> {
    match self {
      Maybe::Ref(r) => Maybe::Ref(r),
      Maybe::Owned(t) => Maybe::Owned(t),
    }
  }

  /// Converts the `Maybe<R, T>` into a `Maybe<&mut R, &mut T>`.
  ///
  /// # Example
  ///
  /// ```
  /// use mayber::Maybe;
  ///
  /// let mut x: Maybe<&str, String> = Maybe::Owned("hello".to_string());
  /// let y = x.as_maybe_mut();
  /// match y {
  ///     Maybe::Owned(s) => s.push_str(" world"),
  ///     Maybe::Ref(_) => panic!("Should be owned"),
  /// }
  /// match &x {
  ///     Maybe::Owned(s) => assert_eq!(s, "hello world"),
  ///     _ => panic!("Should be owned"),
  /// }
  /// ```
  #[inline]
  pub const fn as_maybe_mut(&mut self) -> Maybe<&mut R, &mut T> {
    match self {
      Maybe::Ref(r) => Maybe::Ref(r),
      Maybe::Owned(t) => Maybe::Owned(t),
    }
  }

  /// Maps the contained value to an owned value.
  /// The closure runs only for `Ref`. An `Owned` value moves out directly.
  ///
  /// # Example
  ///
  /// ```
  /// use mayber::MaybeRef;
  ///
  /// fn own_span<T: Clone>(span: MaybeRef<'_, T>) -> T {
  ///     span.map_into_owned(Clone::clone)
  /// }
  ///
  /// let span = 2..5;
  /// assert_eq!(own_span(MaybeRef::Ref(&span)), 2..5);
  /// assert_eq!(own_span(MaybeRef::Owned(span)), 2..5);
  /// ```
  #[inline]
  pub fn map_into_owned(self, f: impl FnOnce(R) -> T) -> T {
    match self {
      Maybe::Ref(r) => f(r),
      Maybe::Owned(t) => t,
    }
  }

  /// Maps the contained value to a reference.
  ///
  /// # Example
  ///
  /// ```
  /// # #[cfg(feature = "std")] {
  /// use mayber::Maybe;
  ///
  /// let x: Maybe<&str, String> = Maybe::Owned("hello".to_string());
  /// let r: &str = x.map_into_ref(|s| s.as_str());
  /// assert_eq!(r, "hello");
  /// # }
  /// ```
  #[inline]
  pub fn map_into_ref<'a>(&'a self, f: impl FnOnce(&'a T) -> R) -> R
  where
    R: Clone,
  {
    match self {
      Maybe::Ref(r) => r.clone(),
      Maybe::Owned(t) => f(t),
    }
  }

  /// Extracts the owned data.
  ///
  /// Clones the data if it is a reference variant, otherwise takes ownership.
  ///
  /// # Example
  ///
  /// ```
  /// use mayber::Maybe;
  ///
  /// let x: Maybe<&str, String> = Maybe::Ref("hello");
  /// let owned: String = x.into_owned();
  /// assert_eq!(&owned, "hello");
  ///
  /// let y: Maybe<&str, String> = Maybe::Owned("world".to_string());
  /// let owned: String = y.into_owned();
  /// assert_eq!(&owned, "world");
  /// ```
  #[inline]
  #[cfg(any(feature = "alloc", feature = "std"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
  pub fn into_owned(self) -> T
  where
    R: Deref,
    R::Target: std::borrow::ToOwned<Owned = T>,
  {
    use std::borrow::ToOwned;

    match self {
      Maybe::Ref(r) => r.deref().to_owned(),
      Maybe::Owned(t) => t,
    }
  }

  /// Creates owned data from borrowed data, always cloning.
  ///
  /// # Example
  ///
  /// ```
  /// use mayber::Maybe;
  ///
  /// let x: Maybe<&str, String> = Maybe::Ref("hello");
  /// let owned: String = x.to_owned();
  /// assert_eq!(owned, "hello");
  /// ```
  #[inline]
  #[cfg(any(feature = "alloc", feature = "std"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
  pub fn to_owned(&self) -> T
  where
    R: Deref,
    R::Target: std::borrow::ToOwned<Owned = T>,
    T: Clone,
  {
    use std::borrow::ToOwned;

    match self {
      Maybe::Ref(r) => r.deref().to_owned(),
      Maybe::Owned(t) => t.clone(),
    }
  }
}

impl<R, T> Maybe<Option<R>, Option<T>> {
  /// Returns `true` if the `Maybe<Option<R>, Option<T>>` is `None`.
  ///
  /// # Example
  ///
  /// ```
  /// use mayber::Maybe;
  ///
  /// let x: Maybe<Option<&str>, Option<String>> = Maybe::Ref(Some("hello"));
  /// assert!(!x.is_none());
  ///
  /// let y: Maybe<Option<&str>, Option<String>> = Maybe::Owned(None);
  /// assert!(y.is_none());
  /// ```
  #[inline]
  pub const fn is_none(&self) -> bool {
    match self {
      Self::Ref(r) => r.is_none(),
      Self::Owned(t) => t.is_none(),
    }
  }

  /// Returns `true` if the `Maybe<Option<R>, Option<T>>` is `Some`.
  ///
  /// # Example
  ///
  /// ```
  /// use mayber::Maybe;
  ///
  /// let x: Maybe<Option<&str>, Option<String>> = Maybe::Ref(Some("hello"));
  /// assert!(x.is_some());
  ///
  /// let y: Maybe<Option<&str>, Option<String>> = Maybe::Owned(None);
  /// assert!(!y.is_some());
  /// ```
  #[inline]
  pub const fn is_some(&self) -> bool {
    match self {
      Self::Ref(r) => r.is_some(),
      Self::Owned(t) => t.is_some(),
    }
  }

  /// Converts the `Maybe<Option<R>, Option<T>>` into a `Maybe<Option<&R>, Option<&T>>`.
  ///
  /// # Example
  ///
  /// ```
  /// use mayber::Maybe;
  ///
  /// let x: Maybe<Option<String>, Option<String>> = Maybe::Owned(Some("hello".to_string()));
  /// let y = x.as_option_ref();
  /// match y {
  ///     Maybe::Owned(Some(s)) => assert_eq!(s, "hello"),
  ///     _ => panic!("Should be Owned(Some)"),
  /// }
  /// ```
  #[inline]
  pub const fn as_option_ref(&self) -> Maybe<Option<&R>, Option<&T>> {
    match self {
      Self::Ref(r) => Maybe::Ref(r.as_ref()),
      Self::Owned(t) => Maybe::Owned(t.as_ref()),
    }
  }

  /// Converts the `Maybe<Option<R>, Option<T>>` into a `Maybe<Option<&mut R>, Option<&mut T>>`.
  ///
  /// # Example
  ///
  /// ```
  /// use mayber::Maybe;
  ///
  /// let mut x: Maybe<Option<String>, Option<String>> = Maybe::Owned(Some("hello".to_string()));
  /// let y = x.as_option_mut();
  /// match y {
  ///     Maybe::Owned(Some(s)) => s.push_str(" world"),
  ///     _ => panic!("Should be Owned(Some)"),
  /// }
  /// match &x {
  ///     Maybe::Owned(Some(s)) => assert_eq!(s, "hello world"),
  ///     _ => panic!("Should be Owned(Some)"),
  /// }
  /// ```
  #[inline]
  pub const fn as_option_mut(&mut self) -> Maybe<Option<&mut R>, Option<&mut T>> {
    match self {
      Self::Ref(r) => Maybe::Ref(r.as_mut()),
      Self::Owned(t) => Maybe::Owned(t.as_mut()),
    }
  }
}

impl<R, T> Maybe<R, T> {
  /// Creates a new `MaybeUninit<Maybe<R, T>>` in an uninitialized state.
  #[inline]
  pub const fn uninit() -> MaybeUninit<Maybe<R, T>> {
    MaybeUninit::uninit()
  }

  /// Creates a new array of `MaybeUninit<Maybe<R, T>>` in an uninitialized state.
  #[inline]
  pub const fn uninit_array<const N: usize>() -> [MaybeUninit<Maybe<R, T>>; N] {
    [const { MaybeUninit::uninit() }; N]
  }

  /// Creates a new `Maybe::Ref` in an uninitialized state for both reference and owned value.
  #[inline]
  pub const fn uninit_both_ref() -> Maybe<MaybeUninit<R>, MaybeUninit<T>> {
    Maybe::Ref(MaybeUninit::uninit())
  }

  /// Creates a new `Maybe::Owned` in an uninitialized state for both reference and owned value.
  #[inline]
  pub const fn uninit_both_owned() -> Maybe<MaybeUninit<R>, MaybeUninit<T>> {
    Maybe::Owned(MaybeUninit::uninit())
  }

  /// Creates a new `Maybe::Ref` in an uninitialized state.
  ///
  /// # Example
  ///
  /// ```
  /// use mayber::Maybe;
  ///
  /// let v: Maybe<std::mem::MaybeUninit<&str>, String> = Maybe::uninit_ref();
  /// ```
  #[inline]
  pub const fn uninit_ref() -> Maybe<MaybeUninit<R>, T> {
    Maybe::Ref(MaybeUninit::uninit())
  }

  /// Creates a new `Maybe::Owned` in an uninitialized state.
  ///
  /// # Example
  ///
  /// ```
  /// use mayber::Maybe;
  ///
  /// let v: Maybe<&str, std::mem::MaybeUninit<String>> = Maybe::uninit_owned();
  /// ```
  #[inline]
  pub const fn uninit_owned() -> Maybe<R, MaybeUninit<T>> {
    Maybe::Owned(MaybeUninit::uninit())
  }
}

impl<R, T> Maybe<MaybeUninit<R>, MaybeUninit<T>> {
  /// Extracts the value from the `MaybeUninit<T>` container. This is a great way
  /// to ensure that the data will get dropped, because the resulting `T` is
  /// subject to the usual drop handling.
  ///
  /// # Safety
  /// The payload of the active variant must be fully initialized and valid for
  /// its type, as required by [`MaybeUninit::assume_init`].
  #[inline]
  pub const unsafe fn assume_init(self) -> Maybe<R, T> {
    match self {
      Self::Ref(r) => Maybe::Ref(r.assume_init()),
      Self::Owned(t) => Maybe::Owned(t.assume_init()),
    }
  }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
const _: () = {
  use std::io::{BufRead, Read, Seek, Write};

  impl<T> Write for MaybeMut<'_, T>
  where
    T: Write,
  {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
      (**self).write(buf)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
      (**self).flush()
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
      (**self).write_vectored(bufs)
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
      (**self).write_all(buf)
    }
    #[inline]
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> std::io::Result<()> {
      (**self).write_fmt(args)
    }
  }

  impl<T> Read for MaybeMut<'_, T>
  where
    T: Read,
  {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
      (**self).read(buf)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
      (**self).read_vectored(bufs)
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
      (**self).read_to_end(buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
      (**self).read_to_string(buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
      (**self).read_exact(buf)
    }
  }

  impl<T> BufRead for MaybeMut<'_, T>
  where
    T: BufRead,
  {
    #[inline]
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
      (**self).fill_buf()
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
      (**self).consume(amt)
    }

    #[inline]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> std::io::Result<usize> {
      (**self).read_until(byte, buf)
    }

    #[inline]
    fn skip_until(&mut self, byte: u8) -> std::io::Result<usize> {
      (**self).skip_until(byte)
    }

    #[inline]
    fn read_line(&mut self, buf: &mut String) -> std::io::Result<usize> {
      (**self).read_line(buf)
    }
  }

  impl<T> Seek for MaybeMut<'_, T>
  where
    T: Seek,
  {
    #[inline]
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
      (**self).seek(pos)
    }

    #[inline]
    fn rewind(&mut self) -> std::io::Result<()> {
      (**self).rewind()
    }

    #[inline]
    fn stream_position(&mut self) -> std::io::Result<u64> {
      (**self).stream_position()
    }

    #[inline]
    fn seek_relative(&mut self, offset: i64) -> std::io::Result<()> {
      (**self).seek_relative(offset)
    }
  }
};

#[cfg(feature = "serde_1")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde_1")))]
const _: () = {
  use serde_core_1::{Deserialize, Deserializer, Serialize, Serializer};

  impl<R, T> Serialize for Maybe<R, T>
  where
    R: Serialize,
    T: Serialize,
  {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      match self {
        Self::Ref(value) => value.serialize(serializer),
        Self::Owned(value) => value.serialize(serializer),
      }
    }
  }

  impl<'de, R, T> Deserialize<'de> for Maybe<R, T>
  where
    T: Deserialize<'de>,
  {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: Deserializer<'de>,
    {
      T::deserialize(deserializer).map(Self::Owned)
    }
  }
};

#[cfg(feature = "borsh_1")]
mod borsh_impl;
#[cfg(any(
  feature = "arbitrary_1",
  feature = "quickcheck_1",
  feature = "proptest_1"
))]
mod generators;
#[cfg(feature = "schemars_1")]
mod schemars_impl;
#[cfg(feature = "rkyv_0_8")]
extern crate rkyv_0_8 as rkyv;
#[cfg(feature = "rkyv_0_8")]
mod rkyv_impl;
#[cfg(feature = "rkyv_0_8")]
pub use rkyv_impl::ArchivedMaybe;
