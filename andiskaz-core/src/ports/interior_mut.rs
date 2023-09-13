use async_trait::async_trait;

use std::ops::{Deref, DerefMut};

pub trait RefCounter: Sized + Clone + Deref {
    fn count(this: &Self) -> usize;
}

pub trait Moving: Sized {
    type Data;

    fn new(data: Self::Data) -> Self;

    fn load(&self) -> Self::Data;

    fn store(&self, data: Self::Data);

    fn swap(&self, data: Self::Data) -> Self::Data;

    fn compare_exchange(&self, old: Self::Data, new: Self::Data) -> Self::Data;
}

#[async_trait]
pub trait Exclusive: Sized {
    type Data;

    type Guard<'a>: DerefMut<Target = Self::Data>
    where
        Self: 'a;

    fn new(data: Self::Data) -> Self;

    async fn acquire<'a>(&'a self) -> Self::Guard<'a>;
}

#[async_trait]
pub trait ReadWrite: Sized {
    type Data;

    type ReadGuard<'a>: Deref<Target = Self::Data>
    where
        Self: 'a;

    type WriteGuard<'a>: DerefMut<Target = Self::Data>
    where
        Self: 'a;

    fn new(data: Self::Data) -> Self;

    async fn acquire_read<'a>(&'a self) -> Self::ReadGuard<'a>;

    async fn acquire_write<'a>(&'a self) -> Self::WriteGuard<'a>;
}
