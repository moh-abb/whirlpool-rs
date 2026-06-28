#![allow(unused)]

use crate::ast::CycleInterval;
use crate::ast::CycleTime;
use crate::interpreter::props::ElemProps;
use crate::interpreter::props::PlayElemArgs;
use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::Index;

struct TotalTimingData;
struct ElemTimingData;

/// Represents a scope that is used by the interpreter during traversal.
/// This could be data that is needed for timing, or variables defined in a
/// local scope or function call, etc.
#[cfg_attr(test, mockall::automock)]
pub trait InterpreterScope {
    #[must_use]
    fn push_total_data(
        &mut self,
        data: TotalTimingData,
    ) -> Result<(), ScopeError>;

    #[must_use]
    fn get_total_data(&self) -> Result<TotalTimingData, ScopeError>;

    #[must_use]
    fn pop_total_data(&mut self) -> Result<TotalTimingData, ScopeError>;

    #[must_use]
    fn push_elem_data(
        &mut self,
        data: ElemTimingData,
    ) -> Result<(), ScopeError>;

    #[must_use]
    fn get_elem_data(&self) -> Result<ElemTimingData, ScopeError>;

    #[must_use]
    fn pop_elem_data(&mut self) -> Result<ElemTimingData, ScopeError>;
}

/// The error type when pushing/popping/accessing a scope item.
#[derive(derive_more::From)]
pub enum ScopeError {
    ArenaErr(ArenaError),
    TypeMismatch,
    EmptyScope,
}

/// Used by [ArenaBackedScope] to allocate the data required for a scope item.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArenaScopeItem {
    item: ScopeItem,
    prev: Option<Index<ArenaScopeItem>>,
}

#[derive(derive_more::TryInto, derive_more::From)]
#[try_into(ref, ref_mut)]
enum ScopeItem {
    TotalTimingData(TotalTimingData),
    ElemTimingData(ElemTimingData),
}

pub struct ArenaBackedScope<'a, A> {
    arena: &'a A,
    last_total_data: Option<Index<ArenaScopeItem>>,
    last_elem_data: Option<Index<ArenaScopeItem>>,
}

impl<'a, A: Arena<ArenaScopeItem>> ArenaBackedScope<'a, A> {
    pub fn new(arena: &'a A) -> Self {
        Self { arena, last_total_data: None, last_elem_data: None }
    }

    fn push<T: Into<ScopeItem>>(
        &mut self,
        item: T,
        get_last_field: fn(&mut Self) -> &mut Option<Index<ArenaScopeItem>>,
    ) -> Result<(), ScopeError> {
        let unit = ArenaScopeItem {
            item: ScopeItem::from(item),
            prev: get_last_field(self).clone(),
        };
        let unit_index = self.arena.push(unit)?;
        *get_last_field(self) = Some(unit_index);
        Ok(())
    }

    fn pop<T: TryFrom<ScopeItem>>(
        &mut self,
        get_last_field: fn(&mut Self) -> &mut Option<Index<ArenaScopeItem>>,
    ) -> Result<T, ScopeError> {
        let unit_index = get_last_field(self)
            .take()
            .ok_or(ScopeError::EmptyScope)?;
        let unit = self.arena.take(unit_index)?;
        let item = unit
            .item
            .try_into()
            .map_err(|_| ScopeError::TypeMismatch)?;
        *get_last_field(self) = unit.prev;
        Ok(item)
    }

    fn get<T: TryFrom<ScopeItem> + Clone>(
        &self,
        get_last_field: fn(&Self) -> &Option<Index<ArenaScopeItem>>,
    ) -> Result<T, ScopeError> {
        let unit_index = get_last_field(self)
            .clone()
            .ok_or(ScopeError::EmptyScope)?;
        let unit = self
            .arena
            .map(unit_index, Clone::clone)?;
        let item = unit
            .item
            .try_into()
            .map_err(|_| ScopeError::TypeMismatch)?;
        Ok(item)
    }
}

impl<'a, A: Arena<ArenaScopeItem>> InterpreterScope
    for ArenaBackedScope<'a, A>
{
    fn push_total_data(
        &mut self,
        data: TotalTimingData,
    ) -> Result<(), ScopeError> {
        self.push(data, |scope| &mut scope.last_total_data)
    }

    fn get_total_data(&self) -> Result<TotalTimingData, ScopeError> {
        self.get(|scope| &scope.last_total_data)
    }

    fn pop_total_data(&mut self) -> Result<TotalTimingData, ScopeError> {
        self.pop(|scope| &mut scope.last_total_data)
    }

    fn push_elem_data(
        &mut self,
        data: ElemTimingData,
    ) -> Result<(), ScopeError> {
        self.push(data, |scope| &mut scope.last_elem_data)
    }

    fn pop_elem_data(&mut self) -> Result<ElemTimingData, ScopeError> {
        self.pop(|scope| &mut scope.last_elem_data)
    }

    fn get_elem_data(&self) -> Result<ElemTimingData, ScopeError> {
        self.get(|scope| &scope.last_total_data)
    }
}
