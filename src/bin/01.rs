use std::collections::HashMap;

advent_of_code::solution!(1);

type LocationId = u32;

pub fn parse(input: &str) -> (Vec<LocationId>, Vec<LocationId>) {
    use itertools::Itertools;

    let location_ids = input
        .split(&['\n', ' '][..])
        .filter_map(|id| id.parse::<LocationId>().ok());
    location_ids.tuples().unzip()
}

pub fn part_one(input: &str) -> Option<u32> {
    let (mut left, mut right) = parse(input);

    // sort both lists to subtract in order
    left.sort_unstable();
    right.sort_unstable();

    let diff = left
        .into_iter()
        .zip(right)
        .fold(0, |acc, (left, right)| acc + left.abs_diff(right));

    Some(diff)
}

pub fn part_two(input: &str) -> Option<u32> {
    let (left, right) = parse(input);

    let right_hist = right.into_iter().fold(HashMap::new(), |mut hist, id| {
        *hist.entry(id).or_default() += 1;
        hist
    });

    let weighted_left = left
        .into_iter()
        .map(|id| id * right_hist.get(&id).unwrap_or(&0));

    Some(weighted_left.sum())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(11));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(31));
    }
}
