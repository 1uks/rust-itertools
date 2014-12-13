//! Licensed under the Apache License, Version 2.0
//! http://www.apache.org/licenses/LICENSE-2.0 or the MIT license
//! http://opensource.org/licenses/MIT, at your
//! option. This file may not be copied, modified, or distributed
//! except according to those terms.

use std::mem;
use std::num::Int;

/// Alternate elements from two iterators until both
/// are run out
///
/// Iterator element type is `A` if `I: Iterator<A>`
#[deriving(Clone)]
pub struct Interleave<I, J> {
    a: I,
    b: J,
    flag: bool,
}

impl<I, J> Interleave<I, J> {
    ///
    pub fn new(a: I, b: J) -> Interleave<I, J> {
        Interleave{a: a, b: b, flag: false}
    }
}

impl<A, I: Iterator<A>, J: Iterator<A>> Iterator<A> for Interleave<I, J> {
    #[inline]
    fn next(&mut self) -> Option<A> {
        self.flag = !self.flag;
        if self.flag {
            match self.a.next() {
                None => self.b.next(),
                r => r,
            }
        } else {
            match self.b.next() {
                None => self.a.next(),
                r => r,
            }
        }
    }
}

/// Clonable iterator adaptor to map elementwise
/// from `Iterator<A>` to `Iterator<B>`
///
/// Created with `.fn_map(..)` on an iterator
///
/// Iterator element type is `B`
pub struct FnMap<A, B, I> {
    map: fn(A) -> B,
    iter: I,
}

impl<A, B, I> FnMap<A, B, I>
{
    pub fn new(iter: I, map: fn(A) -> B) -> FnMap<A, B, I> {
        FnMap{iter: iter, map: map}
    }
}

impl<A, B, I: Iterator<A>> Iterator<B> for FnMap<A, B, I>
{
    #[inline]
    fn next(&mut self) -> Option<B> {
        self.iter.next().map(|a| (self.map)(a))
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<A, B, I: DoubleEndedIterator<A>> DoubleEndedIterator<B>
for FnMap<A, B, I>
{
    #[inline]
    fn next_back(&mut self) -> Option<B> {
        self.iter.next_back().map(|a| (self.map)(a))
    }
}

impl<A, B, I: Clone> Clone for FnMap<A, B, I>
{
    fn clone(&self) -> FnMap<A, B, I> {
        FnMap::new(self.iter.clone(), self.map)
    }
}

/// An iterator adaptor that allows putting back a single
/// item to the front of the iterator.
///
/// Iterator element type is `A`
#[deriving(Clone)]
pub struct PutBack<A, I> {
    top: Option<A>,
    iter: I
}

impl<A, I> PutBack<A, I> {
    /// Iterator element type is `A`
    #[inline]
    pub fn new(it: I) -> PutBack<A, I> {
        PutBack{top: None, iter: it}
    }

    /// Put back a single value to the front of the iterator.
    ///
    /// If a value is already in the put back slot, it is overwritten.
    #[inline]
    pub fn put_back(&mut self, x: A) {
        self.top = Some(x)
    }
}

impl<A, I: Iterator<A>> Iterator<A> for PutBack<A, I> {
    #[inline]
    fn next(&mut self) -> Option<A> {
        match self.top {
            None => self.iter.next(),
            ref mut some => mem::replace(some, None)
        }
    }
    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        let (lo, hi) = self.iter.size_hint();
        match self.top {
            Some(_) => (lo.saturating_add(1), hi.and_then(|x| x.checked_add(1))),
            None => (lo, hi)
        }
    }
}


/// An iterator adaptor that iterates over the cartesian product of
/// the element sets of two iterators `I` and `J`.
///
/// Iterator element type is `(A, B)` if `I: Iterator<A>` and `J: Iterator<B>`
#[deriving(Clone)]
pub struct Product<A, I, J> {
    a: I,
    a_cur: Option<A>,
    b: J,
    b_orig: J,
}

impl<A: Clone, B, I: Iterator<A>, J: Clone + Iterator<B>>
    Product<A, I, J> 
{
    /// Create a new cartesian product iterator
    ///
    /// Iterator element type is `(A, B)` if `I: Iterator<A>` and `J: Iterator<B>`
    pub fn new(i: I, j: J) -> Product<A, I, J>
    {
        let mut i = i;
        Product{a_cur: i.next(), a: i, b: j.clone(), b_orig: j}
    }
}


impl<A: Clone, I: Iterator<A>, B, J: Clone + Iterator<B>>
Iterator<(A, B)> for Product<A, I, J>
{
    fn next(&mut self) -> Option<(A, B)>
    {
        let elt_b = match self.b.next() {
            None => {
                self.b = self.b_orig.clone();
                match self.b.next() {
                    None => return None,
                    Some(x) => {
                        self.a_cur = self.a.next();
                        x
                    }
                }
            }
            Some(x) => x
        };
        match self.a_cur {
            None => None,
            Some(ref a) => {
                Some((a.clone(), elt_b))
            }
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>)
    {
        let (a, ah) = self.a.size_hint();
        let (b, bh) = self.b.size_hint();
        let (bo, boh) = self.b_orig.size_hint();

        // Compute a * bo + b for both lower and upper bound
        let low = a.checked_mul(bo)
                    .and_then(|x| x.checked_add(b))
                    .unwrap_or(::std::uint::MAX);
        let high = ah.and_then(|x| boh.and_then(|y| x.checked_mul(y)))
                     .and_then(|x| bh.and_then(|y| x.checked_add(y)));
        (low, high)
    }
}

/// Remove duplicates from sections of consecutive identical elements.
/// If the iterator is sorted, all elements will be unique.
///
/// Iterator element type is `A` if `I: Iterator<A>`
#[deriving(Clone)]
pub struct Dedup<A, I> {
    last: Option<A>,
    iter: I,
}

impl<A, I> Dedup<A, I>
{
    /// Create a new Dedup Iterator.
    pub fn new(iter: I) -> Dedup<A, I>
    {
        Dedup{last: None, iter: iter}
    }
}

impl<A: Clone + PartialEq, I: Iterator<A>> Iterator<A> for Dedup<A, I>
{
    #[inline]
    fn next(&mut self) -> Option<A>
    {
        for elt in self.iter {
            match self.last {
                Some(ref x) if x == &elt => continue,
                _ => {
                    self.last = Some(elt.clone());
                    return Some(elt)
                }
            }
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>)
    {
        let (lower, upper) = self.iter.size_hint();
        if self.last.is_some() || lower == 0 {
            // they might all be duplicates
            (0, upper)
        } else {
            (1, upper)
        }
    }
}


/// An advanced iterator adaptor. The closure recives a reference to the iterator
/// and may pick off as many elements as it likes, to produce the next iterator element.
///
/// Iterator element type is `B`, if the return type of `F` is `Option<B>`.
#[deriving(Clone)]
pub struct Batching<I, F> {
    f: F,
    iter: I,
}

impl<F, I> Batching<I, F> {
    /// Create a new Batching iterator.
    pub fn new(iter: I, f: F) -> Batching<I, F>
    {
        Batching{f: f, iter: iter}
    }
}

impl<A, B, F: FnMut(&mut I) -> Option<B>, I: Iterator<A>>
    Iterator<B> for Batching<I, F>
{
    #[inline]
    fn next(&mut self) -> Option<B>
    {
        (self.f)(&mut self.iter)
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>)
    {
        // No information about closue behavior
        (0, None)
    }
}
