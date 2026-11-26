use core::borrow::Borrow;
use core::cell::RefCell;

use crate::ast::expr::ExprNode;
use crate::ast::expr::linked::ExprArenas;
use crate::ast::time::CycleTime;
use crate::interpreter::Interpreter;
use crate::interpreter::frame::InterpreterFrame;
use crate::interpreter::pattern::PatternInterpreter;
use crate::mem::Arena;
use crate::mem::GrowableArena;
use crate::mem::Index;
use crate::mem::structures::stack::StackChain;
use crate::synth::unit::SoundUnit;
use crate::test::interpreter::logging::LoggingValueStack;

mod arbitrary;
mod logging;
mod unittests;

/// Performs a simulation on a pattern stored in `arenas` with given head index
/// `head_index`. For each element in `expected_actions`, the first denotes
/// the next time to update, and the second denotes a list of expected scheduled
/// events: the scheduled time, duration, and unit. The expectation will be
/// added and checked sequentially for each pair in `expected_actions`.
fn test_expectations<
    'a,
    'b,
    SoundUnitsAtTime: IntoIterator<Item = impl Borrow<SoundUnit>> + Clone + 'a + 'b,
    SoundUnits: IntoIterator<Item = &'b (CycleTime, SoundUnitsAtTime)> + Clone,
>(
    arenas: &'a impl ExprArenas,
    head_index: Index<ExprNode>,
    expected_sound_units: SoundUnits,
) {
    test_expectations_with_interpreter_setup(
        arenas,
        head_index,
        expected_sound_units,
        EmptyInterpreterSetup,
    )
}

pub trait TestSetupStrategy {
    fn setup_interpreter<
        Arenas,
        ValueStack,
        FrameArena,
        ValueStackBorrow,
        FrameStackBorrow,
    >(
        self,
        interpreter: &mut PatternInterpreter<
            Arenas,
            ValueStack,
            FrameArena,
            ValueStackBorrow,
            FrameStackBorrow,
        >,
    );
}

pub struct FullInterpreterSetup {
    pub offset: CycleTime,
    pub multiplier: CycleTime,
}
impl TestSetupStrategy for FullInterpreterSetup {
    fn setup_interpreter<
        Arenas,
        ValueStack,
        FrameArena,
        ValueStackBorrow,
        FrameArenaBorrow,
    >(
        self,
        interpreter: &mut PatternInterpreter<
            Arenas,
            ValueStack,
            FrameArena,
            ValueStackBorrow,
            FrameArenaBorrow,
        >,
    ) {
        interpreter.set_multiplier(self.multiplier);
        interpreter.set_offset(self.offset);
    }
}

struct EmptyInterpreterSetup;
impl TestSetupStrategy for EmptyInterpreterSetup {
    fn setup_interpreter<
        Arenas,
        ValueStack,
        FrameArena,
        ValueStackBorrow,
        FrameArenaBorrow,
    >(
        self,
        _: &mut PatternInterpreter<
            Arenas,
            ValueStack,
            FrameArena,
            ValueStackBorrow,
            FrameArenaBorrow,
        >,
    ) {
        // Does nothing.
    }
}

fn test_expectations_with_interpreter_setup<
    SoundUnitsAtTime: IntoIterator<Item = impl Borrow<SoundUnit>> + Clone,
    SoundUnits: IntoIterator<Item = impl Borrow<(CycleTime, SoundUnitsAtTime)>> + Clone,
>(
    arenas: &impl ExprArenas,
    head_index: Index<ExprNode>,
    expected_schedule_actions: SoundUnits,
    test_setup: impl TestSetupStrategy,
) {
    test_expectations_with_interpreter_setup_and_start_time(
        arenas,
        head_index,
        CycleTime::ZERO,
        expected_schedule_actions,
        test_setup,
    )
}

fn test_expectations_with_interpreter_setup_and_start_time<
    'a,
    Arenas: ExprArenas,
    SoundUnitsAtTime: IntoIterator<Item = impl Borrow<SoundUnit>> + Clone,
    SoundUnits: IntoIterator<Item = impl Borrow<(CycleTime, SoundUnitsAtTime)>> + Clone,
>(
    arenas: &'a Arenas,
    head_index: Index<ExprNode>,
    start_time: CycleTime,
    expected_schedule_actions: SoundUnits,
    test_setup: impl TestSetupStrategy,
) {
    let value_stack = RefCell::new(LoggingValueStack::new());
    value_stack
        .borrow_mut()
        .set_logging_enabled(false);

    let mut frame_arena = GrowableArena::<StackChain<InterpreterFrame>>::new();
    let frame_arena_refcell = RefCell::new(&mut frame_arena);
    debug_assert_eq!(frame_arena_refcell.borrow().size(), Ok(0));

    let mut interpreter = PatternInterpreter::new(
        head_index,
        arenas,
        &value_stack,
        &frame_arena_refcell,
    );
    test_setup.setup_interpreter(&mut interpreter);

    // Advance the interpreter to the start position.
    // First, ignore all possible played notes before the start position.
    println!("Temporarily disabling value stack");
    value_stack
        .borrow_mut()
        .set_inner_enabled(false);

    interpreter
        .update_time(start_time)
        .unwrap_or_else(|err| {
            panic!("Encountered error when updating start time to {start_time:?}: {err:?}")
        });
    debug_assert_eq!(frame_arena_refcell.borrow().size(), Ok(0));

    println!("Enabling value stack");
    value_stack
        .borrow_mut()
        .set_inner_enabled(true);

    // Enable printing log messages, if desired.
    value_stack
        .borrow_mut()
        .set_logging_enabled(true);

    for borrow_scheduled_actions in expected_schedule_actions {
        let (next_cycle_time, expectations) = borrow_scheduled_actions.borrow();
        expectations
            .clone()
            .into_iter()
            .for_each(|unit| {
                value_stack
                    .borrow_mut()
                    .expect_sound_unit(unit.borrow().clone())
            });

        interpreter
            .update_time(*next_cycle_time)
            .unwrap_or_else(|err| {
                panic!("Encountered error when updating to time {next_cycle_time:?}: {err:?}")
            });

        value_stack.borrow_mut().checkpoint();
    }

    debug_assert_eq!(frame_arena_refcell.borrow().size(), Ok(0));
}
