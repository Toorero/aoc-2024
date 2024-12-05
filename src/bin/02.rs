use itertools::Itertools;

advent_of_code::solution!(2);

type Level = u32;
#[derive(Clone)]
struct Report(Vec<Level>);

impl From<Vec<Level>> for Report {
    fn from(value: Vec<Level>) -> Self {
        Self(value)
    }
}
impl Report {
    pub fn is_safe(&self) -> bool {
        self.is_tolerated_safe(0)
    }

    pub fn is_tolerated_safe(&self, tolerated_failure: usize) -> bool {
        let Self(report) = self;

        let is_increasing = report[0] < report[1];
        let is_save = report.windows(2).all(|slice| {
            let x = slice[0];
            let y = slice[1];

            let abs = x.abs_diff(y);
            let local_increase = x < y;

            // 1. monotonicity
            ! (is_increasing ^ local_increase)
            // 2. level steepness is in between 1..=3
            && (1..=3).contains(&abs)
        });
        if is_save || tolerated_failure == 0 {
            return is_save;
        }

        // try if any removal of a level makes the report save
        (0..report.len())
            .map(|i| {
                let mut report = report.clone();
                report.remove(i);
                Report(report)
            })
            .any(|report| report.is_tolerated_safe(tolerated_failure - 1))
    }
}

fn parse_report_line(line: &str) -> Report {
    let reports: Vec<_> = line
        .split_ascii_whitespace()
        .map(|level| level.parse().unwrap())
        .collect();

    reports.into()
}

fn parse(input: &str) -> Vec<Report> {
    input.lines().map(parse_report_line).collect_vec()
}

pub fn part_one(input: &str) -> Option<usize> {
    let reports = parse(input);

    Some(reports.into_iter().filter(Report::is_safe).count())
}

pub fn part_two(input: &str) -> Option<usize> {
    let reports = parse(input);

    Some(
        reports
            .into_iter()
            .filter(|r| r.is_tolerated_safe(1))
            .count(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(2));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(4));
    }
}
