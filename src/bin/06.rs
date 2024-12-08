use std::collections::HashMap;
use std::collections::HashSet;
use std::iter;
use std::marker::PhantomData;

use itertools::Itertools;

advent_of_code::solution!(6);

pub type Coord = usize;
pub type Move = (i8, i8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SquareState {
    Empty,
    Guard,
    Obstacle,
}

impl TryFrom<char> for SquareState {
    type Error = char;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(match value {
            '.' => Self::Empty,
            '#' => Self::Obstacle,
            '^' => Self::Guard,
            unexpected_char => return Err(unexpected_char),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Default for Direction {
    fn default() -> Self {
        unsafe { Self::iterator().next().unwrap_unchecked() }
    }
}

impl Direction {
    pub fn position_delta(&self) -> Move {
        match self {
            Self::North => (0, -1),
            Self::East => (1, 0),
            Self::South => (0, 1),
            Self::West => (-1, 0),
        }
    }

    pub fn turn_right(&self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }
    pub fn iterator() -> impl Iterator<Item = Self> {
        static DIRECTIONS: [Direction; 4] = [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ];
        DIRECTIONS.iter().copied()
    }
}

#[derive(Debug)]
pub enum PositionError {
    OutOfBounds,
    Obstacle,
    Loop,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Position(pub Coord, pub Coord);

impl Position {
    pub fn try_move(&self, m: Move) -> Result<Self, PositionError> {
        let Self(x, y) = self.clone();
        let (dx, dy) = m;

        let x = x
            .checked_add_signed(dx as _)
            .ok_or(PositionError::OutOfBounds)?;
        let y = y
            .checked_add_signed(dy as _)
            .ok_or(PositionError::OutOfBounds)?;

        Ok(Position(x, y))
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Guard {
    direction: Direction,
    position: Position,
}

impl Guard {
    pub fn new(position: Position, direction: Direction) -> Self {
        Self {
            direction,
            position,
        }
    }

    pub fn with_direction(direction: Direction) -> Self {
        Self {
            direction,
            position: Default::default(),
        }
    }

    pub fn with_position(position: Position) -> Self {
        Self {
            position,
            direction: Default::default(),
        }
    }

    pub fn path<S>(self, area: &Area, step_strategy: &S) -> (Vec<Guard>, PositionError)
    where
        S: StepStrategy<Error = PositionError, Context = Area>,
    {
        let mut guard = self.clone();

        // guard states and final error
        let result_guard_states = iter::from_fn(move || {
            Some(
                area.step_guard(&mut guard, step_strategy)
                    .map(|_| guard.clone()),
            )
        });

        // check for loops
        if let Some((loop_at, _)) = iter::once(Ok(self.clone()))
            .chain(result_guard_states.clone())
            .map_while(Result::ok)
            .enumerate()
            // FIXME: pls don't clone me :/
            .duplicates_by(|(_, guard)| guard.clone())
            .next()
        {
            let guard_states = result_guard_states
                .take(loop_at)
                .map_while(Result::ok)
                .collect();

            (guard_states, PositionError::Loop)
        } else {
            let mut result_guard_states = iter::once(Ok(self)).chain(result_guard_states);

            let guard_states = result_guard_states.by_ref().map_while(Result::ok).collect();
            let err = result_guard_states.next().unwrap().unwrap_err();

            (guard_states, err)
        }
    }
}

pub trait Bounded {
    fn in_bound(&self, position: &Position) -> bool;
}

pub trait Obstructed {
    fn is_obstructed(&self, position: &Position) -> bool;
}

pub trait StepStrategy {
    type Error;
    /// Context for decision making on where to move the guard next
    type Context;

    /// Recommends the next [Position] a [Guard] may take depending on a [Context][Self::Context].
    ///
    /// Never changes a guard on error!
    fn step(&self, guard: &mut Guard, context: &Self::Context) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone)]
pub struct Area {
    width: Coord,
    height: Coord,

    // TODO: Is there a better way to store this?
    //   I think a simple 2d-array would be faster but wastes a ton of space
    obstacles: HashMap<Coord, HashSet<Coord>>,
}

impl Area {
    pub fn new(width: Coord, height: Coord) -> Self {
        Self {
            width,
            height,
            obstacles: Default::default(),
        }
    }

    pub fn add_obstacle(&mut self, pos: Position) -> Result<(), PositionError> {
        if !self.in_bound(&pos) {
            return Err(PositionError::OutOfBounds);
        }

        let Position(x, y) = pos;

        let present = !self.obstacles.entry(x).or_default().insert(y);
        if present {
            return Err(PositionError::Obstacle);
        }

        Ok(())
    }

    pub fn step_guard<S>(&self, guard: &mut Guard, step_strategy: &S) -> Result<(), PositionError>
    where
        S: StepStrategy<Error = PositionError, Context = Self>,
    {
        step_strategy.step(guard, self)
    }
}

impl Bounded for Area {
    fn in_bound(&self, position: &Position) -> bool {
        let Position(x, y) = position;
        x < &self.width && y < &self.height
    }
}

impl Obstructed for Area {
    fn is_obstructed(&self, pos: &Position) -> bool {
        let Position(x, y) = pos;
        self.obstacles.get(x).is_some_and(|line| line.contains(y))
    }
}

#[derive(Debug, Clone)]
pub struct SimpleStepPattern<C> {
    _c: PhantomData<C>,
}

impl<C> Default for SimpleStepPattern<C> {
    fn default() -> Self {
        Self {
            _c: Default::default(),
        }
    }
}

impl<C: Obstructed + Bounded> StepStrategy for SimpleStepPattern<C> {
    type Error = PositionError;
    // obstacles
    type Context = C;

    fn step(&self, guard: &mut Guard, context: &Self::Context) -> Result<(), Self::Error> {
        // only try every direction once for obstacles
        for _d in Direction::iterator() {
            // step in current direction
            let new_pos = guard.position.try_move(guard.direction.position_delta())?;

            // out of bounds!
            if !context.in_bound(&new_pos) {
                return Err(PositionError::OutOfBounds);
            }

            // or turn right on obstruction
            if context.is_obstructed(&new_pos) {
                // NOTE: direction can't be "invalid" so we can change it
                guard.direction = guard.direction.turn_right();
                continue;
            }

            // found position that is 1.) in bound 2.) not obstructed
            guard.position = new_pos;
            return Ok(());
        }

        // if here every direction is obstructed :(
        Err(PositionError::Obstacle)
    }
}

fn charvise(input: &str) -> Vec<Vec<char>> {
    input.lines().map(|line| line.chars().collect()).collect()
}
fn parse_area(input: &str) -> (Area, Vec<Guard>) {
    let lines = charvise(input);

    let width = lines
        .first()
        .map(|first_line| first_line.len())
        .unwrap_or_default();
    let height = lines.len();

    let mut guard = Vec::with_capacity(1);
    let mut area = Area::new(width, height);

    lines.into_iter().zip(0..).for_each(|(line, y)| {
        line.into_iter().zip(0..).for_each(|(square, x)| {
            match SquareState::try_from(square) {
                Ok(SquareState::Empty) => {}
                Ok(SquareState::Obstacle) => area
                    .add_obstacle(Position(x, y))
                    .expect("should be unobstructed because we iterate over each square once"),
                // TODO: support different initial guard directions
                Ok(SquareState::Guard) => guard.push(Guard::with_position(Position(x, y))),
                Err(unexpected_char) => panic!(
                    "Unexpected char '{}'at square ({}, {})",
                    unexpected_char, x, y
                ),
            }
        })
    });

    (area, guard)
}

pub fn part_one(input: &str) -> Option<usize> {
    let strat = SimpleStepPattern::default();
    let (area, mut guards) = parse_area(input);
    let guard = guards.remove(0);

    let (path, err) = guard.path(&area, &strat);
    println!("Stop reason while tracing guard path: {:?}", err);

    let visited = path.into_iter().map(|g| g.position).unique();

    // mark all visited positions visually
    //let mut lines = charvise(input);
    //let visited = visited
    //    .inspect(|Position(x, y)| lines[*y][*x] = 'X')
    //    .count();
    //lines
    //    .into_iter()
    //    .for_each(|line| println!("{}", line.into_iter().collect::<String>()));

    Some(visited.count())
}

pub fn part_two(input: &str) -> Option<usize> {
    let strat = SimpleStepPattern::default();
    let (area, mut guards) = parse_area(input);

    let guard = guards.remove(0);

    // brute force every possible position and check if it creates a loop
    let loop_obst = (0..area.height)
        .flat_map(|y| {
            (0..area.width).map({
                let area = area.clone();
                let guard = guard.clone();
                let strat = &strat;

                move |x| {
                    let mut area = area.clone();

                    let position = Position(x, y);
                    // don't place on guard
                    if position == guard.position {
                        return None;
                    }

                    // add obstacles to temporary area
                    // don't place on existing obstacle
                    if area.add_obstacle(position.clone()).is_err() {
                        return None;
                    }

                    // check if this creates a loop
                    if let (_, PositionError::Loop) = guard.clone().path(&area, strat) {
                        Some(position)
                    } else {
                        None
                    }
                }
            })
        })
        .flatten();

    // mark all visited positions visually
    //let loop_count = loop_obst
    //    .clone()
    //    .inspect(|(l, p)| {
    //        let mut lines = charvise(input);

    //        println!("Found loop at {p:?}:");

    //        let Position(x, y) = *p;
    //        lines[y][x] = 'O';

    //        l.iter().for_each(|g| {
    //            let Position(x, y) = g.position;
    //            lines[y][x] = 'X'
    //        });

    //        lines
    //            .into_iter()
    //            .for_each(|line| println!("{}", line.into_iter().collect::<String>()));
    //    })
    //    .count();

    //let mut lines = charvise(input);
    //let loop_count = loop_obst
    //    .map(|(_, p)| p)
    //    .inspect(|Position(x, y)| lines[*y][*x] = 'o')
    //    .count();
    //lines
    //    .into_iter()
    //    .for_each(|line| println!("{}", line.into_iter().collect::<String>()));

    Some(loop_obst.count())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(41));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(6));
    }
}
