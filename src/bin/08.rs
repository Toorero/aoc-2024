use std::collections::HashMap;
use std::collections::HashSet;
use std::iter;
use std::str::FromStr;
use std::usize;

use itertools::Itertools;
use lina::{Point2, Vec2};

type Coord = Point2<isize>;
type Vector = Vec2<isize>;

advent_of_code::solution!(8);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Frequency {
    Number(u32),
    Lowercase(char),
    Uppercase(char),
}

impl Frequency {
    pub fn new(freq: char) -> Result<Self, &'static str> {
        if freq.is_ascii_digit() {
            return Ok(Self::Number(freq as u32 - '0' as u32));
        }

        if freq.is_ascii_lowercase() {
            return Ok(Self::Lowercase(freq));
        }
        if freq.is_ascii_uppercase() {
            return Ok(Self::Uppercase(freq));
        }

        Err("Unexpected frequency identifier")
    }
}

impl From<u32> for Frequency {
    fn from(value: u32) -> Self {
        Self::Number(value)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Antenna {
    pub frequency: Frequency,
    pub position: Coord,
}

pub enum AntennError {
    SameNode,
    Gradient,
}
impl Antenna {
    pub(crate) fn line_gradient(&self, other: &Antenna) -> Vector {
        other.position.vec_to(self.position)
    }

    pub(crate) fn displace(&self, displacement: Vector) -> Self {
        let Self {
            frequency,
            position,
        } = *self;
        let position = position + displacement;

        Self {
            frequency,
            position,
        }
    }

    pub fn antinode<'a>(
        &'a self,
        other: &'a Antenna,
        bounces: usize,
    ) -> impl Iterator<Item = Antenna> + use<'a> {
        let gradient = self.line_gradient(other);

        assert!(
            !gradient.is_zero(),
            "Calculate antinodes to other antennas only"
        );

        // careful about not going to minus
        let mut self_antinotes = iter::repeat(self).enumerate();
        // if we want all bounces we also include ourselves for some reason
        if bounces != usize::MAX {
            self_antinotes.by_ref().next();
        }
        let self_antinotes = self_antinotes
            .map_while(move |(i, a)| {
                let a = a.displace(gradient.map(|c| c * (i as isize)));

                // check out of bounds
                if a.position[0] >= 0 && a.position[1] >= 0 {
                    Some(a)
                } else {
                    None
                }
            })
            .take(bounces);

        let mut other_antinotes = iter::repeat(other).enumerate();
        // if we want all bounces we also include ourselves for some  reason
        if bounces != usize::MAX {
            other_antinotes.by_ref().next();
        }
        let other_antinotes = other_antinotes
            .map_while(move |(i, a)| {
                let a = a.displace(gradient.map(|c| -c * (i as isize)));

                // check out of bounds
                if a.position[0] >= 0 && a.position[1] >= 0 {
                    Some(a)
                } else {
                    None
                }
            })
            .take(bounces);

        self_antinotes.chain(other_antinotes)
    }
}

#[derive(Debug)]
pub struct AntennaArray<const WIDTH: usize, const HEIGHT: usize> {
    antennas_per_freq: HashMap<Frequency, Vec<Coord>>,
}

impl<const WIDTH: usize, const HEIGHT: usize> AntennaArray<WIDTH, HEIGHT> {
    pub fn antinodes(&self, bounces: usize) -> HashSet<Coord> {
        self.antennas_per_freq
            .iter()
            // calculate antinodes per frequency and collect all positions frequency independent
            .flat_map(|(frequency, antenna_coords)| {
                antenna_coords.iter().combinations(2).flat_map(move |c| {
                    Antenna {
                        frequency: *frequency,
                        position: *c[0],
                    }
                    .antinode(
                        &Antenna {
                            frequency: *frequency,
                            position: *c[1],
                        },
                        bounces,
                    )
                    .map_while(|antinode| {
                        let pos = antinode.position;
                        let x: usize = pos[0].try_into().unwrap();
                        let y: usize = pos[1].try_into().unwrap();

                        // check out of bounds
                        if x < WIDTH && y < HEIGHT {
                            Some(pos)
                        } else {
                            None
                        }
                    })
                    //.filter(|pos| {
                    //    // don't create antinode at antenna positions
                    //    let x: usize = pos[0].try_into().unwrap();
                    //    let y: usize = pos[1].try_into().unwrap();
                    //    !self.antennas[y][x]
                    //})
                    .collect::<Vec<_>>()
                })
            })
            .collect()
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> FromIterator<Antenna>
    for AntennaArray<WIDTH, HEIGHT>
{
    fn from_iter<T: IntoIterator<Item = Antenna>>(iter: T) -> Self {
        let iter = iter.into_iter();

        let mut antennas_per_freq: HashMap<Frequency, Vec<Coord>> =
            HashMap::with_capacity(iter.size_hint().0);

        for antenna in iter {
            let Antenna {
                position,
                frequency,
            } = antenna;
            let x: usize = position[0].try_into().unwrap();
            let y: usize = position[1].try_into().unwrap();

            assert!(x < WIDTH, "Antennas must be in area");
            assert!(y < HEIGHT, "Antennas must be in area");

            antennas_per_freq
                .entry(frequency)
                .or_default()
                .push(Coord::new(x as isize, y as isize));
        }

        Self { antennas_per_freq }
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> FromStr for AntennaArray<WIDTH, HEIGHT> {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let antennas = s.lines().enumerate().flat_map(|(y, line)| {
            line.chars().enumerate().filter_map({
                move |(x, c)| {
                    Frequency::new(c)
                        .map(|frequency| Antenna {
                            frequency,
                            position: Coord::new(x as isize, y as isize),
                        })
                        .ok()
                }
            })
        });

        Ok(Self::from_iter(antennas))
    }
}

fn charvise(input: &str) -> Vec<Vec<char>> {
    input.lines().map(|line| line.chars().collect()).collect()
}

pub fn part_one(input: &str) -> Option<usize> {
    __part_one::<50, 50>(input)
}

fn __part_one<const WIDTH: usize, const HEIGHT: usize>(input: &str) -> Option<usize> {
    part::<WIDTH, HEIGHT>(input, 1)
}

fn part<const WIDTH: usize, const HEIGHT: usize>(input: &str, bounces: usize) -> Option<usize> {
    let arr = AntennaArray::<WIDTH, HEIGHT>::from_str(input).unwrap();

    let antinodes = arr.antinodes(bounces);
    let count = antinodes.len();

    let mut lines = charvise(input);
    antinodes.into_iter().for_each(|pos| {
        let x = pos[0] as usize;
        let y = pos[1] as usize;

        if lines[y][x] == '.' {
            lines[y][x] = '#';
        } else {
            lines[y][x] = '+';
        }
    });
    lines
        .into_iter()
        .for_each(|line| println!("{}", line.into_iter().collect::<String>()));

    Some(count)
}

pub fn part_two(input: &str) -> Option<usize> {
    __part_two::<50, 50>(input)
}

fn __part_two<const WIDTH: usize, const HEIGHT: usize>(input: &str) -> Option<usize> {
    part::<WIDTH, HEIGHT>(input, usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = __part_one::<12, 12>(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(14));
    }

    #[test]
    fn test_part_two() {
        let result = __part_two::<12, 12>(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(34));
    }
}
