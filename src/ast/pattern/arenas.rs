use core::fmt::Debug;

use crate::ast::Pattern;
use crate::ast::TimedStep;
use crate::mem::Arena;
use crate::mem::Chain;

pub trait PatternArenas: Debug {
    fn get_pattern_arena(&self) -> &impl Arena<Pattern>;
    fn get_pattern_chain_arena(&self) -> &impl Arena<Chain<Pattern>>;
    fn get_timed_step_arena(&self) -> &impl Arena<TimedStep>;
    fn get_timed_step_chain_arena(&self) -> &impl Arena<Chain<TimedStep>>;
}
