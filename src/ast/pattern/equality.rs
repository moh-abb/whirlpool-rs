use core::cell::RefCell;
use core::cmp::Ordering;
use core::ops::ControlFlow;

use crate::arena::Arena;
use crate::arena::ArenaItem;
use crate::arena::chain::Chain;
use crate::arena::index::Index;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::pattern_discriminant;
use crate::ast::pattern::visitor::PatternVisitor;
use crate::ast::pattern::visitor::visit_pattern;

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
    with_other: impl FnOnce(&Pattern) -> Ordering,
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
        .then_with(|| with_other(&cloned_target))
}

type ComparisonOutput<Item> = Option<ControlFlow<Ordering, Index<Chain<Item>>>>;

/// Assuming two [Pattern]s have the same discriminant, maps the result of the
/// comparison of a [Chain] of items (e.g., [Pattern::TimeCat] or [Pattern]).
/// Because the discriminant is taken to be the same, the `chain_output` should
/// be `Some` with the value of the chain's comparison. If this value is a
/// [ControlFlow::Continue], then this means that the other chain has more
/// items and so the `this` chain is shorter than the `other` chain, giving
/// [Ordering::Less].
fn cmp_final_chain_output<Item: ArenaItem>(
    chain_output: ComparisonOutput<Item>,
    item_arena: &impl Arena<Chain<Item>>,
) -> Ordering {
    match chain_output.unwrap() {
        // The `this` chain is shorter than, or equal to the `other`.
        ControlFlow::Continue(tail) => {
            // Check whether the `tail` value is `Nil`.
            let tail_is_cons = item_arena
                .inspect(tail, |chain| matches!(chain, Chain::Cons { .. }))
                .unwrap();
            if tail_is_cons { Ordering::Less } else { Ordering::Equal }
        }
        ControlFlow::Break(b) => b,
    }
}

fn fold_comparison_output<Item: ArenaItem>(
    chain_output: ComparisonOutput<Item>,
    other_chain_arena: &impl Arena<Chain<Item>>,
    cmp_items: impl FnOnce(Index<Item>, Index<Item>) -> Ordering,
    this_head: Index<Item>,
) -> ComparisonOutput<Item> {
    let break_with_ordering =
        |ordering: Ordering| Some(ControlFlow::Break(ordering));
    let other_next_chain_index = match chain_output? {
        ControlFlow::Break(b) => return break_with_ordering(b),
        ControlFlow::Continue(index) => index,
    };
    let other_chain = other_chain_arena
        .inspect(other_next_chain_index, Clone::clone)
        .unwrap();
    let Chain::Cons { head: other_head, tail: other_tail } = other_chain else {
        // Because we are in `fold_pattern_chain_output`, our current
        // position in the `this` pattern is a `Cons`, and so we should
        // have the other chain be a `Cons`; otherwise we return
        // `Ordering::Greater`.
        return break_with_ordering(Ordering::Greater);
    };
    let compare_heads = cmp_items(this_head, other_head);
    if compare_heads != Ordering::Equal {
        return break_with_ordering(compare_heads);
    }
    Some(ControlFlow::Continue(other_tail))
}

impl<'a, Arenas1: PatternArenas, Arenas2: PatternArenas> PatternVisitor
    for OrdVisitor<'a, Arenas1, Arenas2>
{
    type Output = Ordering;
    type PatternOutput = Ordering;
    type PatternChainEntry = ();
    type PatternChainOutput = ComparisonOutput<Pattern>;
    type TimedStepOutput = Ordering;
    type TimedStepChainEntry = ();
    type TimedStepChainOutput = ComparisonOutput<TimedStep>;

    const CHAINS_FOLD_RIGHT: bool = false;

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

    fn exit_cat(
        &self,
        _chain_index: Index<Chain<Pattern>>,
        _pattern_chain_entry: Self::PatternChainEntry,
        pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternOutput {
        let inner = self.inner.borrow();
        let cmp_with_other = |other: &Pattern| {
            assert!(matches!(other, Pattern::Cat(_)));
            cmp_final_chain_output(
                pattern_chain_output,
                inner
                    .target_arenas
                    .get_pattern_chain_arena(),
            )
        };
        cmp_patterns_by_discriminant_then_other(
            &inner.source,
            &inner.target,
            self.source_arenas,
            inner.target_arenas,
            cmp_with_other,
        )
    }

    fn exit_seq(
        &self,
        _chain_index: Index<Chain<Pattern>>,
        _pattern_chain_entry: Self::PatternChainEntry,
        pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternOutput {
        let inner = self.inner.borrow();
        let cmp_with_other = |other: &Pattern| {
            assert!(matches!(other, Pattern::Seq(_)));
            cmp_final_chain_output(
                pattern_chain_output,
                inner
                    .target_arenas
                    .get_pattern_chain_arena(),
            )
        };
        cmp_patterns_by_discriminant_then_other(
            &inner.source,
            &inner.target,
            self.source_arenas,
            inner.target_arenas,
            cmp_with_other,
        )
    }

    fn exit_stack(
        &self,
        _chain_index: Index<Chain<Pattern>>,
        _pattern_chain_entry: Self::PatternChainEntry,
        pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternOutput {
        let inner = self.inner.borrow();
        let cmp_with_other = |other: &Pattern| {
            assert!(matches!(other, Pattern::Stack(_)));
            cmp_final_chain_output(
                pattern_chain_output,
                inner
                    .target_arenas
                    .get_pattern_chain_arena(),
            )
        };
        cmp_patterns_by_discriminant_then_other(
            &inner.source,
            &inner.target,
            self.source_arenas,
            inner.target_arenas,
            cmp_with_other,
        )
    }

    fn exit_time_cat(
        &self,
        _chain_index: Index<Chain<TimedStep>>,
        _timed_step_chain_entry: Self::TimedStepChainEntry,
        timed_step_chain_output: Self::TimedStepChainOutput,
    ) -> Self::PatternOutput {
        let inner = self.inner.borrow();
        let cmp_with_other = |other: &Pattern| {
            assert!(matches!(other, Pattern::TimeCat(_)));
            cmp_final_chain_output(
                timed_step_chain_output,
                inner
                    .target_arenas
                    .get_timed_step_chain_arena(),
            )
        };
        cmp_patterns_by_discriminant_then_other(
            &inner.source,
            &inner.target,
            self.source_arenas,
            inner.target_arenas,
            cmp_with_other,
        )
    }

    fn map_note_unit(
        &self,
        unit: super::note::NoteUnit,
    ) -> Self::PatternOutput {
        let inner = self.inner.borrow();
        let cmp_with_other = |other: &Pattern| {
            let Pattern::Note(other_unit) = other else { unreachable!() };
            unit.cmp(other_unit)
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
        let cmp_with_other = |other: &Pattern| {
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

    /// Enters the other pattern's inner pattern chain in tandem with the
    /// visitor (i.e., the other pattern should also include the type
    /// `Index<Chain<Pattern>>`). If the other pattern does not follow this,
    /// then returns `None`. Otherwise, assuming arena accesses are
    /// successful, returns `Some(ControlFlow::Continue(index))`
    /// where `index` is the pattern chain's next index.
    fn new_pattern_chain_output(&self) -> Self::PatternChainOutput {
        // The target should also have an inner Index<Chain<Pattern>>; i.e.
        // be a Cat, Seq or Stack.
        let inner = self.inner.borrow();
        inner
            .target_arenas
            .get_pattern_arena()
            .inspect(inner.target.clone(), |pattern| match pattern {
                Pattern::Cat(index)
                | Pattern::Seq(index)
                | Pattern::Stack(index) => Some(index.clone()),
                Pattern::TimeCat(_) | Pattern::Note(_) | Pattern::Silence => {
                    None
                }
            })
            .unwrap()
            .map(ControlFlow::Continue)
    }

    fn fold_pattern_chain_output(
        &self,
        pattern_chain_output: Self::PatternChainOutput,
        _next_chain_index: Index<Chain<Pattern>>,
        pattern_index: Index<Pattern>,
    ) -> Self::PatternChainOutput {
        let inner = self.inner.borrow();
        let compare_subpatterns = |lhs, rhs| {
            Self::cmp(lhs, rhs, self.source_arenas, inner.target_arenas)
        };
        fold_comparison_output(
            pattern_chain_output,
            inner
                .target_arenas
                .get_pattern_chain_arena(),
            compare_subpatterns,
            pattern_index,
        )
    }

    /// Enters the other pattern's timed step chain in tandem with the
    /// visitor (i.e., the other pattern should also include the type
    /// `Index<Chain<TimedStep>>`). If the other pattern does not follow this,
    /// then returns `None`. Otherwise, assuming arena accesses are
    /// successful, returns `Some(ControlFlow::Continue(index))`
    /// where `index` is the pattern chain's next index.
    fn new_timed_step_chain_output(&self) -> Self::TimedStepChainOutput {
        // The target should also be a `TimeCat`; return `None` otherwise.
        let inner = self.inner.borrow();
        inner
            .target_arenas
            .get_pattern_arena()
            .inspect(inner.target.clone(), |pattern| match pattern {
                Pattern::TimeCat(index) => Some(index.clone()),
                Pattern::Cat(_)
                | Pattern::Seq(_)
                | Pattern::Stack(_)
                | Pattern::Note(_)
                | Pattern::Silence => None,
            })
            .unwrap()
            .map(ControlFlow::Continue)
    }

    fn fold_timed_step_chain_output(
        &self,
        timed_step_chain_output: Self::TimedStepChainOutput,
        _next_chain_index: Index<Chain<TimedStep>>,
        timed_step_index: Index<TimedStep>,
    ) -> Self::TimedStepChainOutput {
        let inner = self.inner.borrow();
        let compare_timed_steps =
            |lhs: Index<TimedStep>, rhs: Index<TimedStep>| {
                // Check the heads for equality; if they are equal,
                // continue with the next tail.
                // INV: the next call to `fold_pattern_chain_output`
                // sees `next_chain_index` equal to this iteration's
                // `_this_tail`.
                let this_timed_step = self
                    .source_arenas
                    .get_timed_step_arena()
                    .inspect(lhs.clone(), Clone::clone)
                    .unwrap();
                let other_timed_step = inner
                    .target_arenas
                    .get_timed_step_arena()
                    .inspect(rhs.clone(), Clone::clone)
                    .unwrap();
                Ord::cmp(&this_timed_step, &other_timed_step)
            };
        fold_comparison_output(
            timed_step_chain_output,
            inner
                .target_arenas
                .get_timed_step_chain_arena(),
            compare_timed_steps,
            timed_step_index,
        )
    }

    fn enter_cat(
        &self,
        _index: Index<Chain<Pattern>>,
    ) -> Self::PatternChainEntry {
    }

    fn enter_seq(
        &self,
        _index: Index<Chain<Pattern>>,
    ) -> Self::PatternChainEntry {
    }

    fn enter_stack(
        &self,
        _index: Index<Chain<Pattern>>,
    ) -> Self::PatternChainEntry {
    }

    fn enter_time_cat(
        &self,
        _index: Index<Chain<TimedStep>>,
    ) -> Self::PatternChainEntry {
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
