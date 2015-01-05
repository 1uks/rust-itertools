#![feature(associated_types)]
#![feature(unboxed_closures)]
#![feature(macro_rules)]
#![crate_name="itertools"]
#![crate_type="dylib"]

//! Itertools — extra iterator adaptors, functions and macros
//!
//! To use the iterator methods in this crate, import the [**Itertools** trait](./trait.Itertools.html):
//!
//! ```ignore
//! use itertools::Itertools;
//! ```
//!
//! Some adaptors are just used directly like regular structs,
//! for example [**PutBack**](./struct.PutBack.html), [**Zip**](./struct.Zip.html), [**Stride**](./struct.Stride.html), [**StrideMut**](./struct.StrideMut.html).
//!
//! To use the macros in this crate, use the `phase(plugin)` attribute:
//!
//! ```ignore
//! #![feature(phase)]
//! #[phase(plugin, link)] extern crate itertools;
//! ```
//!
//! You can shorten the crate name with something like:
//!
//! ```ignore
//! use itertools as it;
//! ```
//! ## License 
//! Dual-licensed to be compatible with the Rust project.
//!
//! Licensed under the Apache License, Version 2.0
//! http://www.apache.org/licenses/LICENSE-2.0 or the MIT license
//! http://opensource.org/licenses/MIT, at your
//! option. This file may not be copied, modified, or distributed
//! except according to those terms.
//!
//!

pub use adaptors::{
    Interleave,
    Product,
    PutBack,
    FnMap,
    Dedup,
    Batching,
    GroupBy,
};
pub use intersperse::Intersperse;
pub use islice::{GenericRange, ISlice};
pub use map::MapMut;
pub use rciter::RcIter;
pub use stride::Stride;
pub use stride::StrideMut;
pub use tee::Tee;
pub use times::Times;
pub use times::times;
pub use linspace::{linspace, Linspace};
pub use zip::{ZipLongest, EitherOrBoth};
pub use ziptuple::Zip;
mod adaptors;
mod intersperse;
mod islice;
mod linspace;
mod map;
mod rciter;
mod stride;
mod tee;
mod times;
mod zip;
mod ziptuple;

/// A helper trait for (x,y,z) ++ w => (x,y,z,w),
/// used for implementing `iproduct!` and `izip!`
#[deprecated]
trait AppendTuple<X, Y> {
    fn append(self, x: X) -> Y;
}

macro_rules! impl_append_tuple(
    () => (
        impl<T> AppendTuple<T, (T, )> for () {
            fn append(self, x: T) -> (T, ) {
                (x, )
            }
        }
    );

    ($A:ident $(,$B:ident)*) => (
        impl_append_tuple!($($B),*);
        #[allow(non_snake_case)]
        impl<$A, $($B,)* T> AppendTuple<T, ($A, $($B,)* T)> for ($A, $($B),*) {
            fn append(self, x: T) -> ($A, $($B,)* T) {
                let ($A, $($B),*) = self;
                ($A, $($B,)* x)
            }
        }
    );
);

impl_append_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);

/// A helper iterator that maps an iterator of tuples like
/// `((A, B), C)` to an iterator of `(A, B, C)`.
///
/// Used by the `izip!()` and `iproduct!()` macros.
#[derive(Clone)]
#[deprecated]
pub struct FlatTuples<I> {
    pub iter: I,
}

impl<X, Y, T: AppendTuple<X, Y>, I>
Iterator for FlatTuples<I>
    where I: Iterator<Item=(T, X)>
{
    type Item = Y;
    #[inline]
    fn next(&mut self) -> Option<Y>
    {
        self.iter.next().map(|(t, x)| t.append(x))
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<X, Y, T: AppendTuple<X, Y>, I: DoubleEndedIterator>
DoubleEndedIterator for FlatTuples<I>
    where I: Iterator<Item=(T, X)>
{
    #[inline]
    fn next_back(&mut self) -> Option<Y>
    {
        self.iter.next_back().map(|(t, x)| t.append(x))
    }
}

#[macro_export]
/// Create an iterator over the “cartesian product” of iterators.
///
/// Iterator element type is like `(A, B, ..., E)` if formed
/// from iterators `(I, J, ..., M)` implementing `I: Iterator<A>`,
/// `J: Iterator<B>`, ..., `M: Iterator<E>`
///
/// ## Example
///
/// ```ignore
/// // Iterate over the coordinates of a 4 x 4 grid
/// // from (0, 0), (0, 1), .. etc until (3, 3)
/// for (i, j) in iproduct!(range(0, 4i), range(0, 4i)) {
///    // ..
/// }
/// ```
pub macro_rules! iproduct(
    ($I:expr) => (
        ($I)
    );
    ($I:expr, $J:expr $(, $K:expr)*) => (
        {
            let it = ::itertools::Product::new($I, $J);
            $(
                let it = ::itertools::FlatTuples{iter: ::itertools::Product::new(it, $K)};
            )*
            it
        }
    );
);

#[deprecated="izip!() is deprecated, use Zip::new instead"]
#[macro_export]
/// *This macro is deprecated, use* **Zip::new** *instead.*
///
/// Create an iterator running multiple iterators in lockstep.
///
/// The izip! iterator yields elements until any subiterator
/// returns `None`.
///
/// Iterator element type is like `(A, B, ..., E)` if formed
/// from iterators `(I, J, ..., M)` implementing `I: Iterator<A>`,
/// `J: Iterator<B>`, ..., `M: Iterator<E>`
///
/// ## Example
///
/// ```ignore
/// // Iterate over three sequences side-by-side
/// let mut xs = [0u, 0, 0];
/// let ys = [72u, 73, 74];
/// for (i, a, b) in izip!(range(0, 100u), xs.mut_iter(), ys.iter()) {
///    *a = i ^ *b;
/// }
/// ```
pub macro_rules! izip(
    ($I:expr) => (
        ($I)
    );
    ($I:expr, $J:expr $(, $K:expr)*) => (
        {
            ::itertools::Zip::new(($I, $J $(, $K)*))
        }
    );
);

// Note: Instead of using struct Product, we could implement iproduct!()
// using .flat_map as well; however it can't implement size_hint.
// ($I).flat_map(|x| Repeat::new(x).zip($J))


/// `icompr` as in “iterator comprehension” allows creating a
/// mapped iterator with simple syntax, similar to set builder notation,
/// and directly inspired by Python. Supports an optional filter clause.
/// 
/// Syntax:
/// 
///  `icompr!(<expression> for <pattern> in <iterator>)`
///
/// or
///
///  `icompr!(<expression> for <pattern> in <iterator> if <expression>)`
///
/// Each element from the `<iterator>` expression is pattern matched
/// with the `<pattern>`, and the bound names are used to express the
/// mapped-to value.
///
/// Iterator element type is the type of `<expression>`
///
/// ## Example
///
/// ```ignore
/// let mut squares = icompr!(x * x for x in range(1i, 100));
/// ```
#[macro_export]
pub macro_rules! icompr(
    ($r:expr for $x:pat in $J:expr if $pred:expr) => (
        ($J).filter_map(|$x| if $pred { Some($r) } else { None })
    );
    ($r:expr for $x:pat in $J:expr) => (
        ($J).filter_map(|$x| Some($r))
    );
);

/// Extra iterator methods for arbitrary iterators
pub trait Itertools : Iterator + Sized {
    // adaptors

    /// Like regular `.map`, but using a simple function pointer instead,
    /// so that the resulting `FnMap` iterator value can be cloned.
    ///
    /// Iterator element type is `B`
    #[deprecated="Use libstd .map() instead"]
    fn fn_map<B>(self, map: fn(<Self as Iterator>::Item) -> B) -> FnMap< <Self as Iterator>::Item, B, Self> {
        FnMap::new(self, map)
    }

    /// Like regular `.map`, but using an unboxed closure instead.
    ///
    /// Iterator element type is `B`
    #[deprecated="Use libstd .map() instead"]
    fn map_unboxed<B, F: FnMut(<Self as Iterator>::Item) -> B>(self, map: F) -> MapMut<F, Self> {
        MapMut::new(self, map)
    }

    /// Alternate elements from two iterators until both
    /// are run out
    ///
    /// Iterator element type is `Self::Item`
    fn interleave<J: Iterator<Item=<Self as Iterator>::Item>>(self, other: J) -> Interleave<Self, J> {
        Interleave::new(self, other)
    }

    /// An iterator adaptor to insert a particular value
    /// between each element of the adapted iterator.
    ///
    /// Iterator element type is `Self::Item`
    fn intersperse(self, element:  <Self as Iterator>::Item) -> Intersperse< <Self as Iterator>::Item, Self> {
        Intersperse::new(self, element)
    }

    /// Creates an iterator which iterates over both this and the specified
    /// iterators simultaneously, yielding pairs of two optional elements.
    /// When both iterators return None, all further invocations of next() will
    /// return None.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use itertools::EitherOrBoth::{Both, Right};
    /// # use itertools::Itertools;
    /// let a = [0i];
    /// let b = [1i, 2i];
    /// let mut it = a.iter().cloned().zip_longest(b.iter().cloned());
    /// assert_eq!(it.next(), Some(Both(0, 1)));
    /// assert_eq!(it.next(), Some(Right(2)));
    /// assert_eq!(it.next(), None);
    /// ```
    ///
    /// Iterator element type is `EitherOrBoth<Self::Item, B>`.
    #[inline]
    fn zip_longest<U: Iterator>(self, other: U) -> ZipLongest<Self, U> {
        ZipLongest::new(self, other)
    }

    /// Remove duplicates from sections of consecutive identical elements.
    /// If the iterator is sorted, all elements will be unique.
    ///
    /// Iterator element type is `Self::Item`.
    fn dedup(self) -> Dedup< <Self as Iterator>::Item, Self> {
        Dedup::new(self)
    }

    /// A “meta iterator adaptor”. Its closure recives a reference to the iterator
    /// and may pick off as many elements as it likes, to produce the next iterator element.
    ///
    /// Iterator element type is `B`.
    ///
    /// ## Example
    ///
    /// ```
    /// # use itertools::Itertools;
    /// let xs = [0i, 1, 2, 1, 3];
    ///
    /// // An adaptor that gathers elements up in pairs
    /// let mut pit = xs.iter().cloned().batching(|mut it| {
    ///            match it.next() {
    ///                None => None,
    ///                Some(x) => match it.next() {
    ///                    None => None,
    ///                    Some(y) => Some((x, y)),
    ///                }
    ///            }
    ///        });
    /// assert_eq!(pit.next(), Some((0, 1)));
    /// assert_eq!(pit.next(), Some((2, 1)));
    /// assert_eq!(pit.next(), None);
    /// ```
    ///
    fn batching<B, F: FnMut(&mut Self) -> Option<B>>(self, f: F) -> Batching<Self, F> {
        Batching::new(self, f)
    }

    /// Group iterator elements. Consecutive elements that map to the same key (“runs”),
    /// are returned as the iterator elements of **GroupBy**.
    ///
    /// Iterator element type is **(K, Vec\<Self::Item\>)**
    fn group_by<K, F: FnMut(& <Self as Iterator>::Item) -> K>(self, key: F) -> GroupBy< <Self as Iterator>::Item, K, Self, F>
    {
        GroupBy::new(self, key)
    }

    /// Split into an iterator pair that both yield all elements from
    /// the original iterator.
    ///
    /// The iterator element `Self::Item` must be clonable.
    ///
    /// Iterator element type is `Self::Item`.
    ///
    /// ## Example
    /// ```
    /// # use itertools::Itertools;
    /// let xs = vec![0i, 1, 2, 3];
    ///
    /// let (mut t1, mut t2) = xs.into_iter().tee();
    /// assert_eq!(t1.next(), Some(0));
    /// assert_eq!(t1.next(), Some(1));
    /// assert_eq!(t2.next(), Some(0));
    /// assert_eq!(t1.next(), Some(2));
    /// assert_eq!(t1.next(), Some(3));
    /// assert_eq!(t1.next(), None);
    /// assert_eq!(t2.next(), Some(1));
    /// ```
    fn tee(self) -> (Tee< <Self as Iterator>::Item, Self>, Tee< <Self as Iterator>::Item, Self>)
    {
        tee::new(self)
    }

    /// Return a sliced iterator.
    ///
    /// **Note:** slicing an iterator is not constant time, and much less efficient than
    /// slicing for example a vector.
    ///
    /// ## Example
    /// ```
    /// # #![feature(slicing_syntax)]
    /// # extern crate itertools;
    /// # fn main() {
    /// use std::iter::repeat;
    /// # use itertools::Itertools;
    ///
    /// let mut it = repeat('a').slice(..3);
    /// assert_eq!(it.count(), 3);
    /// # }
    /// ```
    fn slice<R: GenericRange>(self, range: R) -> ISlice<Self>
    {
        ISlice::new(self, range)
    }

    /// Return an iterator inside a **Rc\<RefCell\<_\>\>** wrapper.
    ///
    /// The returned **RcIter** can be cloned, and each clone will refer back to the
    /// same original iterator.
    ///
    /// **RcIter** allows doing interesting things like using **.zip()** on an iterator with
    /// itself, at the cost of runtime borrow checking.
    /// (If it is not obvious: this has a performance penalty.)
    ///
    /// ## Example
    ///
    /// ```
    /// # use itertools::Itertools;
    ///
    /// let xs = [0i, 1, 1, 1, 2, 1, 3, 5, 6, 7];

    /// let mut rit = xs.iter().cloned().into_rc();
    /// let mut z = rit.clone().zip(rit.clone());
    /// assert_eq!(z.next(), Some((0, 1)));
    /// assert_eq!(z.next(), Some((1, 1)));
    /// assert_eq!(z.next(), Some((2, 1)));
    /// assert_eq!(rit.next(), Some(3));
    /// assert_eq!(z.next(), Some((5, 6)));
    /// assert_eq!(z.next(), None);
    /// ```
    ///
    /// **Panics** in iterator methods if a borrow error is encountered,
    /// but it can only happen if the RcIter is reentered in for example **.next()**,
    /// i.e. if it somehow participates in an "iterator knot" where it is an adaptor of itself.
    ///
    /// Iterator element type is **Self::Item**.
    fn into_rc(self) -> RcIter<Self>
    {
        RcIter::new(self)
    }

    // non-adaptor methods

    /// Consume the first **n** elements of the iterator eagerly.
    ///
    /// Return actual number of elements consumed,
    /// until done or reaching the end.
    fn dropn(&mut self, mut n: uint) -> uint {
        let start = n;
        while n > 0 {
            match self.next() {
                Some(..) => n -= 1,
                None => break
            }
        }
        start - n
    }

    /// Consume the first **n** elements from the iterator eagerly,
    /// and return the same iterator again.
    ///
    /// It works similarly to **.skip(n)** except it is eager and
    /// preserves the iterator type.
    fn dropping(mut self, n: uint) -> Self {
        self.dropn(n);
        self
    }

    /// Run the iterator, eagerly, to the end and consume all its elements.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use itertools::Itertools;
    ///
    /// let mut cnt = 0i;
    /// "hi".chars().map(|c| cnt += 1).drain();
    /// ```
    ///
    fn drain(&mut self) {
        for _ in *self { /* nothing */ }
    }

    /// Run the closure **f** eagerly on each element of the iterator.
    ///
    /// Consumes the iterator until its end.
    fn apply<F: FnMut(<Self as Iterator>::Item)>(&mut self, mut f: F) {
        for elt in *self { f(elt) }
    }

    /// **.collec_vec()** is simply a type specialization of **.collect()**,
    /// for convenience.
    fn collect_vec(self) -> Vec< <Self as Iterator>::Item>
    {
        self.collect()
    }
}

impl<T: Iterator> Itertools for T { }

/// Assign to each reference in `to` from `from`, stopping
/// at the shortest of the two iterators.
///
/// Return the number of elements written.
#[inline]
pub fn write<'a, A: 'a, I: Iterator<Item=&'a mut A>, J: Iterator<Item=A>>
    (mut to: I, mut from: J) -> uint
{
    let mut count = 0u;
    for elt in from {
        match to.next() {
            None => break,
            Some(ptr) => *ptr = elt
        }
        count += 1;
    }
    count
}
