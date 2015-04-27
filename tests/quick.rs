#![cfg_attr(feature = "qc", feature(plugin, custom_attribute))]
#![cfg_attr(feature="qc", plugin(quickcheck_macros))]
#![allow(dead_code)]

//! The purpose of these tests is to cover corner cases of iterators
//! and adaptors.
//!
//! In particular we test the tedious size_hint and exact size correctness.

#[macro_use]
extern crate itertools;

#[cfg(feature = "qc")]
extern crate quickcheck;

#[cfg(feature = "qc")]
mod quicktests {

use quickcheck as qc;
use std::ops::Range;
use itertools;
use itertools::Itertools;
use itertools::{
    Zip,
    Stride,
    EitherOrBoth,
};

/// Our base iterator that we can impl Arbitrary for
#[derive(Clone, Debug)]
pub struct Iter<T>(Range<T>);

impl<T> Iterator for Iter<T> where Range<T>: Iterator
{
    type Item = <Range<T> as Iterator>::Item;
    fn next(&mut self) -> Option<Self::Item> { self.0.next() }
    fn size_hint(&self) -> (usize, Option<usize>)
    {
        self.0.size_hint()
    }
}

impl<T> DoubleEndedIterator for Iter<T> where Range<T>: DoubleEndedIterator
{
    fn next_back(&mut self) -> Option<Self::Item> { self.0.next_back() }
}

impl<T> ExactSizeIterator for Iter<T> where Range<T>: ExactSizeIterator
{ }

impl<T> qc::Arbitrary for Iter<T> where T: qc::Arbitrary
{
    fn arbitrary<G: qc::Gen>(g: &mut G) -> Self
    {
        Iter(T::arbitrary(g)..T::arbitrary(g))
    }

    fn shrink(&self) -> Box<Iterator<Item=Iter<T>>>
    {
        let r = self.0.clone();
        Box::new(
            r.start.shrink().flat_map(move |x| {
                r.end.shrink().map(move |y| (x.clone(), y))
            })
            .map(|(a, b)| Iter(a..b))
        )
    }
}

fn correct_size_hint_fast<I: Iterator>(it: I) -> bool {
    let (low, hi) = it.size_hint();
    let cnt = it.count();
    cnt >= low &&
        (hi.is_none() || hi.unwrap() >= cnt)
}

fn correct_size_hint<I: Iterator>(mut it: I) -> bool {
    // record size hint at each iteration
    let initial_hint = it.size_hint();
    let mut hints = Vec::with_capacity(initial_hint.0 + 1);
    hints.push(initial_hint);
    while let Some(_) = it.next() {
        hints.push(it.size_hint())
    }

    let mut true_count = hints.len(); // start off +1 too much

    // check all the size hints
    for &(low, hi) in &hints {
        true_count -= 1;
        if low > true_count ||
            (hi.is_some() && hi.unwrap() < true_count)
        {
            println!("True size: {:?}, size hint: {:?}", true_count, (low, hi));
            println!("All hints: {:?}", hints);
            return false
        }
    }
    true
}

fn exact_size<I: ExactSizeIterator>(mut it: I) -> bool {
    // check every iteration
    let (mut low, mut hi) = it.size_hint();
    if Some(low) != hi { return false; }
    while let Some(_) = it.next() {
        let (xlow, xhi) = it.size_hint();
        if low != xlow + 1 { return false; }
        low = xlow;
        hi = xhi;
        if Some(low) != hi { return false; }
    }
    let (low, hi) = it.size_hint();
    low == 0 && hi == Some(0)
}

/*
 * NOTE: Range<i8> is broken!
 * (all signed ranges are)
#[quickcheck]
fn size_range_i8(a: Iter<i8>) -> bool {
    exact_size(a)
}

#[quickcheck]
fn size_range_i16(a: Iter<i16>) -> bool {
    exact_size(a)
}

#[quickcheck]
fn size_range_u8(a: Iter<u8>) -> bool {
    exact_size(a)
}
 */

#[quickcheck]
fn size_stride(data: Vec<u8>, mut stride: isize) -> bool {
    if stride == 0 {
        stride += 1; // never zero
    }
    exact_size(Stride::from_slice(&data, stride))
}

#[quickcheck]
fn equal_stride(data: Vec<u8>, mut stride: i8) -> bool {
    if stride == 0 {
        // never zero
        stride += 1;
    }
    if stride > 0 {
        itertools::equal(Stride::from_slice(&data, stride as isize),
                         data.iter().step(stride as usize))
    } else {
        itertools::equal(Stride::from_slice(&data, stride as isize),
                         data.iter().rev().step(-stride as usize))
    }
}

#[quickcheck]
fn size_product(a: Iter<u16>, b: Iter<u16>) -> bool {
    correct_size_hint(a.cartesian_product(b))
}

#[quickcheck]
fn size_product3(a: Iter<u16>, b: Iter<u16>, c: Iter<u16>) -> bool {
    correct_size_hint(iproduct!(a, b, c))
}

#[quickcheck]
fn size_step(a: Iter<u16>, mut s: usize) -> bool {
    if s == 0 {
        s += 1; // never zero
    }
    exact_size(a.step(s))
}

#[quickcheck]
fn size_multipeek(a: Iter<u16>, s: u8) -> bool {
    let mut it = a.multipeek();
    // peek a few times
    for _ in 0..s {
        it.peek();
    }
    exact_size(it)
}

#[quickcheck]
fn equal_merge(a: Vec<i16>, b: Vec<i16>) -> bool {
    let mut sa = a.clone();
    let mut sb = b.clone();
    sa.sort();
    sb.sort();
    let mut merged = sa.clone() + &sb;
    merged.sort();
    itertools::equal(&merged, sa.iter().merge(&sb))

}
#[quickcheck]
fn size_merge(a: Iter<u16>, b: Iter<u16>) -> bool {
    correct_size_hint(a.merge(b))
}

#[quickcheck]
fn size_zip(a: Iter<i16>, b: Iter<i16>, c: Iter<i16>) -> bool {
    exact_size(Zip::new((a, b, c)))
}

#[quickcheck]
fn size_zip_longest(a: Iter<i16>, b: Iter<i16>) -> bool {
    exact_size(a.zip_longest(b))
}

#[quickcheck]
fn size_2_zip_longest(a: Iter<i16>, b: Iter<i16>) -> bool {
    let it = a.clone().zip_longest(b.clone());
    let jt = a.clone().zip_longest(b.clone());
    itertools::equal(a.clone(),
                     it.filter_map(|elt| match elt {
                         EitherOrBoth::Both(x, _) => Some(x),
                         EitherOrBoth::Left(x) => Some(x),
                         _ => None,
                     }
                     ))
        &&
    itertools::equal(b.clone(),
                     jt.filter_map(|elt| match elt {
                         EitherOrBoth::Both(_, y) => Some(y),
                         EitherOrBoth::Right(y) => Some(y),
                         _ => None,
                     }
                     ))
}

fn equal_islice(a: Vec<i16>, x: usize, y: usize) -> bool {
    if x > y || y > a.len() { return true; }
    let slc = &a[x..y];
    itertools::equal(a.iter().slice(x..y), slc)
}

#[quickcheck]
fn size_interleave(a: Iter<i16>, b: Iter<i16>) -> bool {
    correct_size_hint(a.interleave(b))
}

#[quickcheck]
fn size_intersperse(a: Iter<i16>, x: i16) -> bool {
    correct_size_hint(a.intersperse(x))
}

#[quickcheck]
fn equal_intersperse(a: Vec<i32>, x: i32) -> bool {
    let mut inter = false;
    let mut i = 0;
    for elt in a.iter().cloned().intersperse(x) {
        if inter {
            if elt != x { return false }
        } else {
            if elt != a[i] { return false }
            i += 1;
        }
        inter = !inter;
    }
    true
}

#[quickcheck]
fn equal_dedup(a: Vec<i32>) -> bool {
    let mut b = a.clone();
    b.dedup();
    itertools::equal(&b, a.iter().dedup())
}

#[quickcheck]
fn size_dedup(a: Vec<i32>) -> bool {
    correct_size_hint(a.iter().dedup())
}

#[quickcheck]
fn size_group_by(a: Vec<i8>) -> bool {
    correct_size_hint(a.iter().group_by(|x| x.abs()))
}

#[quickcheck]
fn size_linspace(a: f32, b: f32, n: usize) -> bool {
    let it = itertools::linspace(a, b, n);
    it.len() == n &&
        exact_size(it)
}

#[quickcheck]
fn equal_repeatn(n: usize, x: i32) -> bool {
    let it = itertools::RepeatN::new(x, n);
    exact_size(it)
}

#[cfg(feature = "unstable")]
#[quickcheck]
fn size_ziptrusted(a: Vec<u8>, b: Vec<u8>) -> bool {
    exact_size(itertools::ZipTrusted::new((a.iter(), b.iter())))
}

#[cfg(feature = "unstable")]
#[quickcheck]
fn size_ziptrusted3(a: Vec<u8>, b: Vec<u8>, c: Vec<u8>) -> bool {
    exact_size(itertools::ZipTrusted::new((a.iter(), b.iter(), c.iter())))
}

#[cfg(feature = "unstable")]
#[quickcheck]
fn equal_ziptrusted_mix(a: Vec<u8>, b: Vec<()>, x: u8, y: u8) -> bool {
    let it = itertools::ZipTrusted::new((a.iter(), b.iter(), x..y));
    let jt = Zip::new((a.iter(), b.iter(), x..y));
    itertools::equal(it, jt)
}

#[cfg(feature = "unstable")]
#[quickcheck]
fn size_ziptrusted_mix(a: Vec<u8>, b: Vec<()>, x: u8, y: u8) -> bool {
    exact_size(itertools::ZipTrusted::new((a.iter(), b.iter(), x..y)))
}

}
