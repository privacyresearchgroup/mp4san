#![allow(missing_docs)]

use std::fmt::Debug;
use std::marker::PhantomData;

use bytes::{Buf, BufMut, BytesMut};
use derive_where::derive_where;
use mediasan_common::error::WhileParsingType;
use mediasan_common::ResultExt;

use crate::error::Result;

use super::{Mp4Prim, Mp4Value, Mp4ValueWriterExt, ParseError};

#[derive(PartialEq, Eq)]
#[derive_where(Clone, Debug, Default; C)]
pub struct BoundedArray<C, T> {
    entry_count: C,
    array: UnboundedArray<T>,
}

#[derive(PartialEq, Eq)]
#[derive_where(Clone, Debug, Default)]
pub struct UnboundedArray<T> {
    entries: BytesMut,
    _t: PhantomData<T>,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct ArrayEntry<'a, T> {
    data: &'a [u8],
    _t: PhantomData<T>,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct ArrayEntryMut<'a, T> {
    data: &'a mut [u8],
    _t: PhantomData<T>,
}

//
// BoundedArray impls
//

impl<C: Clone, T: Mp4Prim> BoundedArray<C, T> {
    pub fn entries(&self) -> impl Iterator<Item = ArrayEntry<'_, T>> + ExactSizeIterator + '_ {
        self.array.entries()
    }

    pub fn entries_mut(&mut self) -> impl Iterator<Item = ArrayEntryMut<'_, T>> + ExactSizeIterator + '_ {
        self.array.entries_mut()
    }

    pub fn entry_count(&self) -> C {
        self.entry_count.clone()
    }
}

impl<C: Mp4Prim + Into<u32> + Clone, T: Mp4Prim> Mp4Value for BoundedArray<C, T> {
    fn parse(buf: &mut BytesMut) -> Result<Self, ParseError> {
        let entry_count = C::parse(&mut *buf).while_parsing_type()?;
        let entries_len = (T::ENCODED_LEN as u32)
            .checked_mul(entry_count.clone().into())
            .ok_or_else(|| report_attach!(ParseError::InvalidInput, "overflow", WhileParsingType::new::<Self>()))?;
        ensure_attach!(
            buf.remaining() as u32 >= entries_len,
            ParseError::TruncatedBox,
            WhileParsingType::new::<Self>(),
        );
        let mut array_bytes = buf.split_to(entries_len as usize);
        let array = UnboundedArray::parse(&mut array_bytes)?;
        Ok(Self { entry_count, array })
    }

    fn encoded_len(&self) -> u64 {
        C::ENCODED_LEN + self.array.encoded_len()
    }

    fn put_buf<B: BufMut>(&self, mut buf: B) {
        buf.put_mp4_value(&self.entry_count);
        buf.put_slice(&self.array.entries);
    }
}

impl<C: From<u32>, T: Mp4Prim> FromIterator<T> for BoundedArray<C, T> {
    fn from_iter<I: IntoIterator<Item = T>>(entries: I) -> Self {
        let array = UnboundedArray::from_iter(entries);
        Self { entry_count: (array.entry_count() as u32).into(), array }
    }
}

//
// UnboundedArray impls
//

impl<T: Mp4Prim> UnboundedArray<T> {
    pub fn entries(&self) -> impl Iterator<Item = ArrayEntry<'_, T>> + ExactSizeIterator + '_ {
        self.entries
            .chunks_exact(T::ENCODED_LEN as usize)
            .map(|data| ArrayEntry { data, _t: PhantomData })
    }

    pub fn entries_mut(&mut self) -> impl Iterator<Item = ArrayEntryMut<'_, T>> + ExactSizeIterator + '_ {
        self.entries
            .chunks_exact_mut(T::ENCODED_LEN as usize)
            .map(|data| ArrayEntryMut { data, _t: PhantomData })
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len() / T::ENCODED_LEN as usize
    }
}

impl<T: Mp4Prim> Mp4Value for UnboundedArray<T> {
    fn parse(buf: &mut BytesMut) -> Result<Self, ParseError> {
        let entries = buf.split();
        Ok(Self { entries, _t: PhantomData })
    }

    fn encoded_len(&self) -> u64 {
        self.entries.len() as u64
    }

    fn put_buf<B: BufMut>(&self, mut buf: B) {
        buf.put_slice(&self.entries[..])
    }
}

impl<T: Mp4Prim> FromIterator<T> for UnboundedArray<T> {
    fn from_iter<I: IntoIterator<Item = T>>(entries: I) -> Self {
        let mut entries_bytes = BytesMut::new();
        for entry in entries {
            entry.put_buf(&mut entries_bytes);
        }
        Self { entries: entries_bytes, _t: PhantomData }
    }
}

//
// ArrayEntry impls
//

impl<T: Mp4Prim> ArrayEntry<'_, T> {
    pub fn get(&self) -> Result<T, ParseError> {
        T::parse(self.data)
    }
}

//
// ArrayEntryMut impls
//

impl<T: Mp4Prim> ArrayEntryMut<'_, T> {
    pub fn get(&self) -> Result<T, ParseError> {
        T::parse(&*self.data)
    }

    pub fn set(&mut self, value: T) {
        self.data.put_mp4_value(&value)
    }
}
