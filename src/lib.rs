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
  borrow::Borrow,
  fmt,
  mem::MaybeUninit,
  ops::{Deref, DerefMut},
};

mod ref_or_val_sealed {
  pub trait Sealed<T> {}
}

/// A trait that allows abstracting over references to or owned values of a `T`.
///
/// This trait is sealed: you cannot implement it yourself.
pub trait RefOrVal<'a, T: 'a>:
  ref_or_val_sealed::Sealed<T> + Borrow<T> + Into<MaybeRef<'a, T>>
{
  /// Project the referential properties of this type onto another type.
  ///
  /// For example, `<&Foo>::Proj<Bar> = &Bar` but `<Foo>::Proj<Bar> = Bar`.
  #[doc(hidden)]
  type Proj<U: 'a>;

  /// The target type when dereferencing.
  #[doc(hidden)]
  type DerefTarget: ?Sized;

  #[doc(hidden)]
  fn map_maybe<R: 'a>(
    self,
    f: impl FnOnce(&'a T) -> &'a R,
    g: impl FnOnce(T) -> R,
  ) -> Self::Proj<R>;

  #[doc(hidden)]
  fn as_deref_target(&self) -> &Self::DerefTarget;
}

impl<T> ref_or_val_sealed::Sealed<T> for &T {}
impl<'a, T> RefOrVal<'a, T> for &'a T {
  type Proj<U: 'a> = &'a U;
  type DerefTarget = T;

  fn map_maybe<R: 'a>(
    self,
    f: impl FnOnce(&'a T) -> &'a R,
    _g: impl FnOnce(T) -> R,
  ) -> Self::Proj<R> {
    f(self)
  }

  fn as_deref_target(&self) -> &Self::DerefTarget {
    self
  }
}

impl<T> ref_or_val_sealed::Sealed<T> for T {}
impl<'a, T: 'a> RefOrVal<'a, T> for T
where
  T: Deref,
{
  type Proj<U: 'a> = U;
  type DerefTarget = T::Target;

  fn map_maybe<R: 'a>(
    self,
    _f: impl FnOnce(&'a T) -> &'a R,
    g: impl FnOnce(T) -> R,
  ) -> Self::Proj<R> {
    g(self)
  }

  fn as_deref_target(&self) -> &Self::DerefTarget {
    self.deref()
  }
}

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
///     println!("Length: {}", s.deref().len());
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
///     s.deref_mut().push_str(" world");
/// }
///
/// let mut owned = String::from("hello");
/// modify(MaybeMut::Owned(owned));
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
pub enum Maybe<R, T> {
  /// A reference to a value
  Ref(R),
  /// An owned value
  Owned(T),
}

/// A helper type that implements `Deref<Target = T>` for `Maybe<R, T>` where `R: Deref<Target = T>`.
///
/// This allows `Maybe<&T, T>` (where T doesn't implement Deref) to access T's methods through
/// deref coercion. Obtain this type by calling [`Maybe::deref()`].
pub struct MaybeDeref<'a, R, T>(&'a Maybe<R, T>);

impl<R, T> Deref for MaybeDeref<'_, R, T>
where
  R: Deref<Target = T>,
{
  type Target = T;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref(&self) -> &T {
    match self.0 {
      Maybe::Ref(r) => r.deref(),
      Maybe::Owned(t) => t,
    }
  }
}

/// A helper type that implements `DerefMut<Target = T>` for `Maybe<R, T>` where `R: DerefMut<Target = T>`.
///
/// Obtain this type by calling [`Maybe::deref_mut()`].
pub struct MaybeDerefMut<'a, R, T>(&'a mut Maybe<R, T>);

impl<R, T> Deref for MaybeDerefMut<'_, R, T>
where
  R: Deref<Target = T>,
{
  type Target = T;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref(&self) -> &T {
    match &*self.0 {
      Maybe::Ref(r) => r.deref(),
      Maybe::Owned(t) => t,
    }
  }
}

impl<R, T> DerefMut for MaybeDerefMut<'_, R, T>
where
  R: DerefMut<Target = T>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref_mut(&mut self) -> &mut T {
    match &mut *self.0 {
      Maybe::Ref(r) => r.deref_mut(),
      Maybe::Owned(t) => t,
    }
  }
}

// impl<R, T> Borrow<T::Target> for Maybe<R, T>
// where
//   R: Deref<Target = <T as Deref>::Target>,
//   T: Deref,
// {
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   fn borrow(&self) -> &T::Target {
//     self
//   }
// }

// impl<R, T> BorrowMut<T::Target> for &mut Maybe<R, T>
// where
//   R: DerefMut<Target = <T as Deref>::Target>,
//   T: DerefMut,
// {
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   fn borrow_mut(&mut self) -> &mut T::Target {
//     self
//   }
// }

impl<R, T> AsRef<T::Target> for Maybe<R, T>
where
  R: Deref<Target = <T as Deref>::Target>,
  T: Deref,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn as_ref(&self) -> &T::Target {
    self
  }
}

impl<R, T> AsMut<T::Target> for Maybe<R, T>
where
  R: DerefMut<Target = <T as Deref>::Target>,
  T: DerefMut,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn as_mut(&mut self) -> &mut T::Target {
    self
  }
}

impl<R, T> Deref for Maybe<R, T>
where
  R: Deref<Target = <T as Deref>::Target>,
  T: Deref,
{
  type Target = <T as Deref>::Target;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref(&self) -> &Self::Target {
    match self {
      Maybe::Ref(r) => r.deref(),
      Maybe::Owned(t) => t,
    }
  }
}

impl<R, T> DerefMut for Maybe<R, T>
where
  R: DerefMut<Target = <T as Deref>::Target>,
  T: DerefMut,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref_mut(&mut self) -> &mut Self::Target {
    match self {
      Maybe::Ref(r) => r.deref_mut(),
      Maybe::Owned(t) => t,
    }
  }
}

impl<R, T> PartialEq for Maybe<R, T>
where
  R: Deref<Target = T>,
  T: PartialEq,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Maybe::Ref(a), Maybe::Ref(b)) => a.deref().eq(b.deref()),
      (Maybe::Ref(a), Maybe::Owned(b)) => a.deref().eq(b),
      (Maybe::Owned(a), Maybe::Ref(b)) => a.eq(b.deref()),
      (Maybe::Owned(a), Maybe::Owned(b)) => a.eq(b),
    }
  }
}

impl<R, T, O> PartialEq<&O> for Maybe<R, T>
where
  R: Deref<Target = T>,
  T: PartialEq<O>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &&O) -> bool {
    match self {
      Maybe::Ref(r) => r.deref().eq(*other),
      Maybe::Owned(t) => t.eq(*other),
    }
  }
}

impl<R, T> Eq for Maybe<R, T>
where
  R: Deref<Target = T>,
  T: Eq,
{
}

impl<R, T> PartialOrd for Maybe<R, T>
where
  R: Deref<Target = T>,
  T: PartialOrd,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
    match (self, other) {
      (Maybe::Ref(a), Maybe::Ref(b)) => a.deref().partial_cmp(b.deref()),
      (Maybe::Ref(a), Maybe::Owned(b)) => a.deref().partial_cmp(b),
      (Maybe::Owned(a), Maybe::Ref(b)) => a.partial_cmp(b.deref()),
      (Maybe::Owned(a), Maybe::Owned(b)) => a.partial_cmp(b),
    }
  }
}

impl<R, T, O> PartialOrd<&O> for Maybe<R, T>
where
  R: Deref<Target = T>,
  T: PartialOrd<O>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &&O) -> Option<core::cmp::Ordering> {
    match self {
      Maybe::Ref(r) => r.deref().partial_cmp(*other),
      Maybe::Owned(t) => t.partial_cmp(*other),
    }
  }
}

impl<R, T> Ord for Maybe<R, T>
where
  R: Deref<Target = T>,
  T: Ord,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn cmp(&self, other: &Self) -> core::cmp::Ordering {
    match (self, other) {
      (Maybe::Ref(a), Maybe::Ref(b)) => a.deref().cmp(b.deref()),
      (Maybe::Ref(a), Maybe::Owned(b)) => a.deref().cmp(b),
      (Maybe::Owned(a), Maybe::Ref(b)) => a.cmp(b.deref()),
      (Maybe::Owned(a), Maybe::Owned(b)) => a.cmp(b),
    }
  }
}

impl<R, T> fmt::Display for Maybe<R, T>
where
  R: Deref<Target = T>,
  T: fmt::Display,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Maybe::Ref(r) => r.deref().fmt(f),
      Maybe::Owned(t) => t.fmt(f),
    }
  }
}

impl<R, T> fmt::Debug for Maybe<R, T>
where
  R: Deref<Target = T>,
  T: fmt::Debug,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Maybe::Ref(r) => r.deref().fmt(f),
      Maybe::Owned(t) => t.fmt(f),
    }
  }
}

impl<R, T> Clone for Maybe<R, T>
where
  R: Clone,
  T: Clone,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn clone(&self) -> Self {
    match self {
      Maybe::Ref(r) => Maybe::Ref(r.clone()),
      Maybe::Owned(t) => Maybe::Owned(t.clone()),
    }
  }
}

impl<R, T> Copy for Maybe<R, T>
where
  R: Copy,
  T: Copy,
{
}

impl<R, T> core::hash::Hash for Maybe<R, T>
where
  R: Deref<Target = T>,
  T: core::hash::Hash,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
    match self {
      Maybe::Ref(r) => r.deref().hash(state),
      Maybe::Owned(t) => t.hash(state),
    }
  }
}

impl<R, T> Default for Maybe<R, T>
where
  T: Default,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Maybe::Owned(T::default())
  }
}

impl<R, T> From<T> for Maybe<R, T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(value: T) -> Self {
    Maybe::Owned(value)
  }
}

impl<'a, T> From<&'a T> for Maybe<&'a T, T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(value: &'a T) -> Self {
    Maybe::Ref(value)
  }
}

impl<'a, T> From<&'a mut T> for Maybe<&'a mut T, T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(value: &'a mut T) -> Self {
    Maybe::Ref(value)
  }
}

impl<T> Maybe<&T, T> {
  /// Converts the `Maybe<&T, T>` into a `&T`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_ref(&self) -> &T {
    match self {
      Maybe::Ref(r) => r,
      Maybe::Owned(t) => t,
    }
  }
}

impl<T> Maybe<&mut T, T> {
  /// Converts the `Maybe<&mut T, T>` into a `&mut T`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_mut(&mut self) -> &mut T {
    match self {
      Maybe::Ref(r) => r,
      Maybe::Owned(t) => t,
    }
  }
}

impl<R, T> Maybe<R, T> {
  /// Returns `true` if the Maybe contains a reference
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_ref(&self) -> bool {
    matches!(self, Maybe::Ref(_))
  }

  /// Returns `true` if the Maybe contains an owned value
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_owned(&self) -> bool {
    matches!(self, Maybe::Owned(_))
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
  #[cfg_attr(not(tarpaulin), inline(always))]
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
  #[cfg_attr(not(tarpaulin), inline(always))]
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
  #[cfg_attr(not(tarpaulin), inline(always))]
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
  #[cfg_attr(not(tarpaulin), inline(always))]
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
  #[cfg_attr(not(tarpaulin), inline(always))]
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
  #[cfg_attr(not(tarpaulin), inline(always))]
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
  #[cfg_attr(not(tarpaulin), inline(always))]
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
  #[cfg_attr(not(tarpaulin), inline(always))]
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
  #[cfg_attr(not(tarpaulin), inline(always))]
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
  #[cfg_attr(not(tarpaulin), inline(always))]
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
  #[cfg_attr(not(tarpaulin), inline(always))]
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
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_maybe_mut(&mut self) -> Maybe<&mut R, &mut T> {
    match self {
      Maybe::Ref(r) => Maybe::Ref(r),
      Maybe::Owned(t) => Maybe::Owned(t),
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
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_owned(self) -> T
  where
    R: Deref,
    R::Target: ToOwned<Owned = T>,
  {
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
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn to_owned(&self) -> T
  where
    R: Deref,
    T: Deref,
    R::Target: ToOwned<Owned = T>,
    T::Target: ToOwned<Owned = T>,
  {
    match self {
      Maybe::Ref(r) => r.deref().to_owned(),
      Maybe::Owned(t) => t.deref().to_owned(),
    }
  }

  /// Returns a helper type that implements `Deref<Target = T>` for `Maybe<R, T>`
  /// where `R: Deref<Target = T>`.
  ///
  /// This allows `Maybe<&T, T>` (where T doesn't implement Deref) to still access
  /// T's methods through deref coercion.
  ///
  /// # Example
  ///
  /// ```
  /// use mayber::Maybe;
  ///
  /// #[derive(Clone)]
  /// struct Lexed {
  ///     value: String,
  /// }
  ///
  /// impl Lexed {
  ///     fn get_value(&self) -> &str {
  ///         &self.value
  ///     }
  /// }
  ///
  /// let lexed = Lexed { value: "hello".to_string() };
  /// let x: Maybe<&Lexed, Lexed> = Maybe::Ref(&lexed);
  /// // Access methods through deref()
  /// assert_eq!(x.deref().get_value(), "hello");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn deref(&self) -> MaybeDeref<'_, R, T>
  where
    R: Deref<Target = T>,
  {
    MaybeDeref(self)
  }

  /// Returns a helper type that implements `DerefMut<Target = T>` for `Maybe<R, T>`
  /// where `R: DerefMut<Target = T>`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn deref_mut(&mut self) -> MaybeDerefMut<'_, R, T>
  where
    R: DerefMut<Target = T>,
  {
    MaybeDerefMut(self)
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
  #[cfg_attr(not(tarpaulin), inline(always))]
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
  #[cfg_attr(not(tarpaulin), inline(always))]
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
  #[cfg_attr(not(tarpaulin), inline(always))]
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
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[rustversion::attr(since(1.83), const)]
  pub fn as_option_mut(&mut self) -> Maybe<Option<&mut R>, Option<&mut T>> {
    match self {
      Self::Ref(r) => Maybe::Ref(r.as_mut()),
      Self::Owned(t) => Maybe::Owned(t.as_mut()),
    }
  }
}

impl<R, T> Maybe<R, T> {
  /// Creates a new `MaybeUninit<Maybe<R, T>>` in an uninitialized state.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn uninit() -> MaybeUninit<Maybe<R, T>> {
    MaybeUninit::uninit()
  }

  /// Creates a new array of `MaybeUninit<Maybe<R, T>>` in an uninitialized state.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn uninit_array<const N: usize>() -> [MaybeUninit<Maybe<R, T>>; N] {
    [const { MaybeUninit::uninit() }; N]
  }

  /// Creates a new `Maybe::Ref` in an uninitialized state for both reference and owned value.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn uninit_both_ref() -> Maybe<MaybeUninit<R>, MaybeUninit<T>> {
    Maybe::Ref(MaybeUninit::uninit())
  }

  /// Creates a new `Maybe::Owned` in an uninitialized state for both reference and owned value.
  #[cfg_attr(not(tarpaulin), inline(always))]
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
  #[cfg_attr(not(tarpaulin), inline(always))]
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
  #[cfg_attr(not(tarpaulin), inline(always))]
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
  /// See [`MaybeUninit::assume_init`](core::mem::MaybeUninit::assume_init).
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[rustversion::attr(since(1.59), const)]
  pub unsafe fn assume_init(self) -> Maybe<R, T> {
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

  impl<R, T> Write for Maybe<R, T>
  where
    R: DerefMut<Target = T>,
    T: Write,
  {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
      match self {
        Maybe::Ref(r) => r.deref_mut().write(buf),
        Maybe::Owned(t) => t.write(buf),
      }
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn flush(&mut self) -> std::io::Result<()> {
      match self {
        Maybe::Ref(r) => r.deref_mut().flush(),
        Maybe::Owned(t) => t.flush(),
      }
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
      match self {
        Maybe::Ref(r) => r.deref_mut().write_vectored(bufs),
        Maybe::Owned(t) => t.write_vectored(bufs),
      }
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
      match self {
        Maybe::Ref(r) => r.deref_mut().write_all(buf),
        Maybe::Owned(t) => t.write_all(buf),
      }
    }
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> std::io::Result<()> {
      match self {
        Maybe::Ref(r) => r.deref_mut().write_fmt(args),
        Maybe::Owned(t) => t.write_fmt(args),
      }
    }
  }

  impl<R, T> Read for Maybe<R, T>
  where
    R: DerefMut<Target = T>,
    T: Read,
  {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
      match self {
        Maybe::Ref(r) => r.deref_mut().read(buf),
        Maybe::Owned(t) => t.read(buf),
      }
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
      match self {
        Maybe::Ref(r) => r.deref_mut().read_vectored(bufs),
        Maybe::Owned(t) => t.read_vectored(bufs),
      }
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
      match self {
        Maybe::Ref(r) => r.deref_mut().read_to_end(buf),
        Maybe::Owned(t) => t.read_to_end(buf),
      }
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
      match self {
        Maybe::Ref(r) => r.deref_mut().read_to_string(buf),
        Maybe::Owned(t) => t.read_to_string(buf),
      }
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
      match self {
        Maybe::Ref(r) => r.deref_mut().read_exact(buf),
        Maybe::Owned(t) => t.read_exact(buf),
      }
    }
  }

  impl<R, T> BufRead for Maybe<R, T>
  where
    R: DerefMut<Target = T>,
    T: BufRead,
  {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
      match self {
        Maybe::Ref(r) => r.deref_mut().fill_buf(),
        Maybe::Owned(t) => t.fill_buf(),
      }
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn consume(&mut self, amt: usize) {
      match self {
        Maybe::Ref(r) => r.deref_mut().consume(amt),
        Maybe::Owned(t) => t.consume(amt),
      }
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> std::io::Result<usize> {
      match self {
        Maybe::Ref(r) => r.deref_mut().read_until(byte, buf),
        Maybe::Owned(t) => t.read_until(byte, buf),
      }
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    #[rustversion::since(1.83)]
    fn skip_until(&mut self, byte: u8) -> std::io::Result<usize> {
      match self {
        Maybe::Ref(r) => r.deref_mut().skip_until(byte),
        Maybe::Owned(t) => t.skip_until(byte),
      }
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn read_line(&mut self, buf: &mut String) -> std::io::Result<usize> {
      match self {
        Maybe::Ref(r) => r.deref_mut().read_line(buf),
        Maybe::Owned(t) => t.read_line(buf),
      }
    }
  }

  impl<R, T> Seek for Maybe<R, T>
  where
    R: DerefMut<Target = T>,
    T: Seek,
  {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
      match self {
        Maybe::Ref(r) => r.deref_mut().seek(pos),
        Maybe::Owned(t) => t.seek(pos),
      }
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn rewind(&mut self) -> std::io::Result<()> {
      match self {
        Maybe::Ref(r) => r.deref_mut().rewind(),
        Maybe::Owned(t) => t.rewind(),
      }
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn stream_position(&mut self) -> std::io::Result<u64> {
      match self {
        Maybe::Ref(r) => r.deref_mut().stream_position(),
        Maybe::Owned(t) => t.stream_position(),
      }
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    #[rustversion::since(1.80)]
    fn seek_relative(&mut self, offset: i64) -> std::io::Result<()> {
      match self {
        Maybe::Ref(r) => r.deref_mut().seek_relative(offset),
        Maybe::Owned(t) => t.seek_relative(offset),
      }
    }
  }
};

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
const _: () = {
  use serde_core::{Deserialize, Deserializer, Serialize, Serializer};

  impl<R, T> Serialize for Maybe<R, T>
  where
    R: Deref<Target = T>,
    T: Serialize,
  {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      match self {
        Maybe::Ref(r) => r.deref().serialize(serializer),
        Maybe::Owned(t) => t.serialize(serializer),
      }
    }
  }

  impl<'de, R, T> Deserialize<'de> for Maybe<R, T>
  where
    T: Deserialize<'de>,
  {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: Deserializer<'de>,
    {
      T::deserialize(deserializer).map(Self::Owned)
    }
  }
};
