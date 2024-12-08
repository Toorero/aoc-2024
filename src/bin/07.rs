use std::str::FromStr;
use strum::EnumIter;
use strum::IntoEnumIterator;

advent_of_code::solution!(7);

pub trait BinaryFold<T> {
    fn apply(&self, x: T, y: T) -> T;
}

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum SimpleOperations {
    Addition,
    Multiplication,
}

impl BinaryFold<u64> for SimpleOperations {
    fn apply(&self, x: u64, y: u64) -> u64 {
        match self {
            Self::Addition => x + y,
            Self::Multiplication => x * y,
        }
    }
}

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum AdvancedOperations {
    Addition,
    Multiplication,
    Concatination,
}

impl BinaryFold<u64> for AdvancedOperations {
    fn apply(&self, x: u64, y: u64) -> u64 {
        match self {
            Self::Addition => x + y,
            Self::Multiplication => x * y,
            Self::Concatination => {
                let digits_y = y.ilog10() + 1;
                x * 10u64.pow(digits_y) + y
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct OwnedEquation {
    result: u64,
    numbers: Vec<u64>,
}

impl OwnedEquation {
    pub fn new(result: u64, numbers: Vec<u64>) -> Self {
        assert!(numbers.len() >= 2, "Need two number for an equation");

        Self { result, numbers }
    }

    pub fn is_satisfyable<B: BinaryFold<u64> + IntoEnumIterator>(&self) -> bool {
        let Self { result, numbers } = self;
        let eq = Equation {
            result: *result,
            numbers: &numbers[..],
        };

        eq.is_satisfyable::<B>(0)
    }
}

impl FromStr for OwnedEquation {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((result, numbers)) = s.split_once(':') else {
            return Err("No delimiter ':' between result and numbers found!");
        };

        let result = result.parse().expect("Result should be an u64");
        let words = numbers.split_ascii_whitespace();
        // FIXME: Error instead of panic
        let numbers: Vec<_> = words.map(|w| w.parse().unwrap()).collect();

        if numbers.len() < 2 {
            return Err("Need two numbers for an equation");
        }

        Ok(Self::new(result, numbers))
    }
}

#[derive(Debug, Clone, Copy)]
struct Equation<'a> {
    result: u64,
    numbers: &'a [u64],
}

impl<'a> Equation<'a> {
    pub fn is_satisfyable<B: BinaryFold<u64> + IntoEnumIterator>(
        &self,
        interim_result: u64,
    ) -> bool {
        let Self { result, numbers } = *self;

        for op in B::iter() {
            let interim_result = op.apply(interim_result, numbers[0]);
            // all hopes lost if operation makes it larger than required result
            if interim_result <= result {
                // interim_result is final result on end
                if numbers.len() == 1 {
                    if interim_result == result {
                        return true;
                    }

                    // maybe another operation will save the day
                    continue;
                }

                // else try applying operation recursively for the left numbers
                let numbers = &numbers[1..];
                let reduced_equation = Equation { result, numbers };
                if reduced_equation.is_satisfyable::<B>(interim_result) {
                    return true;
                }

                // else backtrack
            }
        }

        false
    }
}

pub fn part_one(input: &str) -> Option<u64> {
    let lines = input.lines();

    Some(
        lines
            .map(|l| l.parse().unwrap())
            .filter(OwnedEquation::is_satisfyable::<SimpleOperations>)
            .map(|satisfyable_eq| satisfyable_eq.result)
            .sum(),
    )
}

pub fn part_two(input: &str) -> Option<u64> {
    let lines = input.lines();

    Some(
        lines
            .map(|l| l.parse().unwrap())
            .filter(OwnedEquation::is_satisfyable::<AdvancedOperations>)
            .map(|satisfyable_eq| satisfyable_eq.result)
            .sum(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(3749));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(11387));
    }
}
