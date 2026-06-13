use crate::ast::CycleInterval;
use crate::ast::CycleTime;

/// A struct used to denote the simulation properties of each element in a
/// [Multiple]. This includes the element itself, the number of cycles to play
/// it for (its simulated duration), and its played duration.
///
/// The simulated duration is the number of cycles for which to simulate the
/// element. This is usually 1, with the exception of [Pattern::Arrange], which
/// is simulated for the number of cycles in the [TimedStep].
///
/// Note that the played duration is its duration before dividing by the total
/// length of the [Multiple]; i.e. the sequence is fast (e.g., for a
/// [Pattern::Seq], the duration would be 1, not 1 / length). The played
/// duration is usually equal to the simulated duration, with the exception of
/// [Pattern::TimeCat], for which it is
/// `timed_step_duration * multiple_length / total_length`, because each element
/// is played for a full cycle (hence all elements are simulated over
/// `multiple_length` cycles), but needs to be scaled according to the length
/// in the [TimedStep] giving a factor of `timed_step_duration / total_length`.
pub struct ElemProps<T> {
    pub elem: T,
    pub sim_duration: CycleTime,
    pub played_duration: CycleTime,
}

/// Represents the arguments that are passed into the `play_elem` function in
/// [play_elements].
#[derive(Debug)]
pub struct PlayElemArgs<'a, T> {
    pub elem: &'a T,
    pub interval: CycleInterval,
    pub offset: CycleTime,
    pub multiplier: CycleTime,
}
