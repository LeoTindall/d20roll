use futures::Future;
use futures::future::{lazy, ok, err};
use rfyl;

/// The outcome of a roll, tagged with the description of the roll that generated it.
#[derive(Debug)]
pub struct RollOutcome {
    pub descriptor: String,
    pub outcome: i32,
}

pub fn lazy_roll(s: String) -> Box<Future<Item = RollOutcome, Error = ()>> {
    Box::new(lazy(|| {
        let rolls = match rfyl::roll(s) {
            Ok(v) => v,
            Err(_) => return err(())
        };
        ok( RollOutcome { descriptor: rolls.get_rolls_formula_string_as_infix(), outcome: rolls.get_result() }) 
    }))
}