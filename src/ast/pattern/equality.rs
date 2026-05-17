use core::cell::RefCell;
use core::cmp::Ordering;

use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::pattern_discriminant;
use crate::ast::pattern::visitor::PatternVisitor;
use crate::ast::pattern::visitor::visit_pattern;
use crate::mem::arena::Arena;
use crate::mem::arena::ArenaItem;
use crate::structures::chain::Chain;
use crate::structures::index::Index;
use crate::structures::multiple::Multiple;

pub struct OrdVisitor<'a, Arenas1, Arenas2> {
    source_arenas: &'a Arenas1,
    inner: RefCell<OrdVisitorInner<'a, Arenas2>>,
}

struct OrdVisitorInner<'a, Arenas2> {
    source: Index<Pattern>,
    target: Index<Pattern>,
    target_arenas: &'a Arenas2,
}

impl<'a, Arenas1: PatternArenas, Arenas2: PatternArenas>
    OrdVisitor<'a, Arenas1, Arenas2>
{
    fn cmp(
        this: Index<Pattern>,
        other: Index<Pattern>,
        this_arenas: &'a Arenas1,
        other_arenas: &'a Arenas2,
    ) -> Ordering {
        let visitor = Self {
            source_arenas: this_arenas,
            inner: RefCell::new(OrdVisitorInner {
                source: this.clone(),
                target: other,
                target_arenas: other_arenas,
            }),
        };
        visit_pattern(&visitor, this)
    }
}

/// Directly compares two patterns (i.e., not a deep comparison) by their
/// discriminant, then with the function `with_other` provided on the pattern.
fn cmp_patterns_by_discriminant_then_other(
    this: &Index<Pattern>,
    other: &Index<Pattern>,
    this_arenas: &impl PatternArenas,
    other_arenas: &impl PatternArenas,
    with_other: impl FnOnce(Pattern) -> Ordering,
) -> Ordering {
    let source_discriminant = this_arenas
        .get_pattern_arena()
        .inspect(this.clone(), pattern_discriminant)
        .unwrap();
    let cloned_target = other_arenas
        .get_pattern_arena()
        .inspect(other.clone(), Clone::clone)
        .unwrap();
    Ord::cmp(&source_discriminant, &pattern_discriminant(&cloned_target))
        .then_with(|| with_other(cloned_target))
}

fn cmp_multiple<Item: ArenaItem>(
    this_multiple: Multiple<Item>,
    other_multiple: Multiple<Item>,
    this_chain_arena: &impl Arena<Chain<Item>>,
    other_chain_arena: &impl Arena<Chain<Item>>,
    mut cmp_items: impl FnMut(Index<Item>, Index<Item>) -> Ordering,
) -> Ordering {
    // First, compare the multiple items by their lengths.
    let this_length = this_multiple.length();
    let other_length = other_multiple.length();
    if this_length != other_length {
        return this_length.cmp(&other_length);
    }
    // Note that by the precondition above, either both are empty or both are
    // nonempty.
    if this_multiple.is_empty() {
        return Ordering::Equal;
    }
    let this_start = this_multiple.start_end().unwrap().0;
    let other_start = other_multiple.start_end().unwrap().0;
    let mut cur_this_chain = this_start;
    let mut cur_other_chain = other_start;
    loop {
        let this_chain = this_chain_arena
            .inspect(cur_this_chain, Clone::clone)
            .unwrap();
        let other_chain = other_chain_arena
            .inspect(cur_other_chain, Clone::clone)
            .unwrap();
        let Chain(this_index, _this_prev, this_next) = this_chain;
        let Chain(other_index, _other_prev, other_next) = other_chain;
        let item_ordering = cmp_items(this_index, other_index);
        if item_ordering != Ordering::Equal {
            return item_ordering;
        }
        let Some(nexts) = this_next.zip(other_next) else {
            break Ordering::Equal;
        };
        (cur_this_chain, cur_other_chain) = nexts;
    }
}

fn cmp_multiple_pattern<Arenas1: PatternArenas, Arenas2: PatternArenas>(
    visitor: &OrdVisitor<'_, Arenas1, Arenas2>,
    multiple: Multiple<Pattern>,
    mut get_other_multiple: impl FnMut(Pattern) -> Multiple<Pattern>,
) -> Ordering {
    let inner = visitor.inner.borrow();
    let source_chain_arena = visitor
        .source_arenas
        .get_pattern_chain_arena();
    let target_chain_arena = inner
        .target_arenas
        .get_pattern_chain_arena();
    let cmp_subpatterns =
        |sub_source: Index<Pattern>, sub_target: Index<Pattern>| {
            let visitor = OrdVisitor {
                source_arenas: visitor.source_arenas,
                inner: RefCell::new(OrdVisitorInner {
                    source: sub_source.clone(),
                    target: sub_target.clone(),
                    target_arenas: inner.target_arenas,
                }),
            };
            visit_pattern(&visitor, sub_source)
        };
    let cmp_with_other = |other: Pattern| {
        cmp_multiple(
            multiple,
            get_other_multiple(other),
            source_chain_arena,
            target_chain_arena,
            cmp_subpatterns,
        )
    };
    cmp_patterns_by_discriminant_then_other(
        &inner.source,
        &inner.target,
        visitor.source_arenas,
        inner.target_arenas,
        cmp_with_other,
    )
}

fn cmp_multiple_timed_step<Arenas1: PatternArenas, Arenas2: PatternArenas>(
    visitor: &OrdVisitor<'_, Arenas1, Arenas2>,
    multiple: Multiple<TimedStep>,
) -> Ordering {
    let inner = visitor.inner.borrow();
    let source_arena = visitor
        .source_arenas
        .get_timed_step_arena();
    let target_arena = inner
        .target_arenas
        .get_timed_step_arena();
    let source_chain_arena = visitor
        .source_arenas
        .get_timed_step_chain_arena();
    let target_chain_arena = inner
        .target_arenas
        .get_timed_step_chain_arena();
    let get_timed_steps =
        |this_timed_step: Index<TimedStep>,
         other_timed_step: Index<TimedStep>| {
            let cloned_source = source_arena
                .inspect(this_timed_step, Clone::clone)
                .unwrap();
            let cloned_target = target_arena
                .inspect(other_timed_step, Clone::clone)
                .unwrap();
            (cloned_source, cloned_target)
        };
    let cmp_timed_steps =
        |this_timed_step: Index<TimedStep>,
         other_timed_step: Index<TimedStep>| {
            let (cloned_source, cloned_target) =
                get_timed_steps(this_timed_step, other_timed_step);
            let cmp_time_units = cloned_source.0.cmp(&cloned_target.0);
            if cmp_time_units != Ordering::Equal {
                return cmp_time_units;
            }
            let TimedStep(_, source_pattern) = cloned_source;
            let TimedStep(_, target_pattern) = cloned_target;
            let visitor = OrdVisitor {
                source_arenas: visitor.source_arenas,
                inner: RefCell::new(OrdVisitorInner {
                    source: source_pattern.clone(),
                    target: target_pattern,
                    target_arenas: inner.target_arenas,
                }),
            };
            visit_pattern(&visitor, source_pattern)
        };
    let cmp_with_other = |other: Pattern| {
        let (Pattern::TimeCat(other_multiple)
        | Pattern::Arrange(other_multiple)) = other
        else {
            unreachable!()
        };
        cmp_multiple(
            multiple,
            other_multiple,
            source_chain_arena,
            target_chain_arena,
            cmp_timed_steps,
        )
    };
    cmp_patterns_by_discriminant_then_other(
        &inner.source,
        &inner.target,
        visitor.source_arenas,
        inner.target_arenas,
        cmp_with_other,
    )
}

impl<'a, Arenas1: PatternArenas, Arenas2: PatternArenas> PatternVisitor
    for OrdVisitor<'a, Arenas1, Arenas2>
{
    type Output = Ordering;
    type PatternOutput = Ordering;

    fn get_arenas(&self) -> &impl PatternArenas {
        self.source_arenas
    }

    fn map_pattern(
        &self,
        pattern_index: Index<Pattern>,
        pattern_output: Self::PatternOutput,
    ) -> Self::Output {
        assert!(self.inner.borrow().source == pattern_index);
        pattern_output
    }

    fn map_cat(&self, multiple: Multiple<Pattern>) -> Self::PatternOutput {
        cmp_multiple_pattern(self, multiple, |other: Pattern| {
            let Pattern::Cat(other_multiple) = other else { unreachable!() };
            other_multiple
        })
    }

    fn map_seq(&self, multiple: Multiple<Pattern>) -> Self::PatternOutput {
        cmp_multiple_pattern(self, multiple, |other: Pattern| {
            let Pattern::Seq(other_multiple) = other else { unreachable!() };
            other_multiple
        })
    }

    fn map_stack(&self, multiple: Multiple<Pattern>) -> Self::PatternOutput {
        cmp_multiple_pattern(self, multiple, |other: Pattern| {
            let Pattern::Stack(other_multiple) = other else { unreachable!() };
            other_multiple
        })
    }

    fn map_time_cat(
        &self,
        multiple: Multiple<TimedStep>,
    ) -> Self::PatternOutput {
        cmp_multiple_timed_step(self, multiple)
    }

    fn map_arrange(
        &self,
        multiple: Multiple<TimedStep>,
    ) -> Self::PatternOutput {
        cmp_multiple_timed_step(self, multiple)
    }

    fn map_note_unit(
        &self,
        unit: super::note::NoteUnit,
    ) -> Self::PatternOutput {
        let inner = self.inner.borrow();
        let cmp_with_other = |other: Pattern| {
            let Pattern::Note(other_unit) = other else { unreachable!() };
            unit.cmp(&other_unit)
        };
        cmp_patterns_by_discriminant_then_other(
            &inner.source,
            &inner.target,
            self.source_arenas,
            inner.target_arenas,
            cmp_with_other,
        )
    }

    fn map_silence(&self) -> Self::PatternOutput {
        // Only need to compare by the discriminant.
        let inner = self.inner.borrow();
        let cmp_with_other = |other: Pattern| {
            assert!(matches!(other, Pattern::Silence));
            Ordering::Equal
        };
        cmp_patterns_by_discriminant_then_other(
            &inner.source,
            &inner.target,
            self.source_arenas,
            inner.target_arenas,
            cmp_with_other,
        )
    }
}

impl<'a, Arenas1: PatternArenas, Arenas2: PatternArenas> Ord
    for PatternOrdAdapter<'a, Arenas1, Arenas2>
{
    fn cmp(&self, other: &Self) -> Ordering {
        let i1 = self.index.clone();
        let i2 = other.index.clone();
        match (self.arenas, other.arenas) {
            (Ok(a1), Ok(a2)) => OrdVisitor::cmp(i1, i2, a1, a2),
            (Ok(a1), Err(a2)) => OrdVisitor::cmp(i1, i2, a1, a2),
            (Err(a1), Ok(a2)) => OrdVisitor::cmp(i1, i2, a1, a2),
            (Err(a1), Err(a2)) => OrdVisitor::cmp(i1, i2, a1, a2),
        }
    }
}

/// Default implementation of PartialEq for `Index<Pattern>`'s adapter
impl<'a, Arenas1: PatternArenas, Arenas2: PatternArenas> PartialEq
    for PatternOrdAdapter<'a, Arenas1, Arenas2>
{
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

/// Default implementation of [Eq] for `Index<Pattern>`'s adapter.
impl<'a, Arenas1: PatternArenas, Arenas2: PatternArenas> Eq
    for PatternOrdAdapter<'a, Arenas1, Arenas2>
{
}

/// Default implementation of [PartialOrd] for `Index<Pattern>`'s adapter.
impl<'a, Arenas1: PatternArenas, Arenas2: PatternArenas> PartialOrd
    for PatternOrdAdapter<'a, Arenas1, Arenas2>
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
pub struct PatternOrdAdapter<'a, Arenas1, Arenas2> {
    index: Index<Pattern>,
    arenas: Result<&'a Arenas1, &'a Arenas2>,
}

impl<'a, Arenas1: PatternArenas, Arenas2: PatternArenas>
    PatternOrdAdapter<'a, Arenas1, Arenas2>
{
    #[allow(unused)]
    pub fn new_left(index: Index<Pattern>, arenas: &'a Arenas1) -> Self {
        Self { index, arenas: Ok(arenas) }
    }

    #[allow(unused)]
    pub fn new_right(index: Index<Pattern>, arenas: &'a Arenas2) -> Self {
        Self { index, arenas: Err(arenas) }
    }
}

impl<'a, Arenas: PatternArenas> PatternOrdAdapter<'a, Arenas, Arenas> {
    #[allow(unused)]
    pub fn new(index: Index<Pattern>, arenas: &'a Arenas) -> Self {
        Self { index, arenas: Ok(arenas) }
    }
}
