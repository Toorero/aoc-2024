use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;

advent_of_code::solution!(5);

type PageNumber = u32;
#[derive(Default, Debug)]
struct PageConstraints {
    before: HashSet<PageNumber>,
    after: HashSet<PageNumber>,
}

#[derive(Default, Debug)]
struct UpdateConstraints(HashMap<PageNumber, PageConstraints>);

impl UpdateConstraints {
    pub fn constraint(&mut self, before: PageNumber, after: PageNumber) {
        // insert both implied constraints
        let _ = self.0.entry(before).or_default().before.insert(after);
        let _ = self.0.entry(after).or_default().after.insert(before);

        // check afterwards that we don't have conflicting constraints x|y and y|x
        // this is crucial for the recursion of required_print_ord to end
        debug_assert!({
            let before_constraints = self.0.get(&before).unwrap();
            before_constraints
                .before
                .intersection(&before_constraints.after)
                .next()
                .is_none()
        },);
    }

    pub fn required_print_ord(&self, x: &PageNumber, y: &PageNumber) -> Ordering {
        let Some(x_constraints) = self.0.get(x) else {
            return Ordering::Equal;
        };

        if x_constraints.before.contains(y) {
            Ordering::Less
        } else if x_constraints.after.contains(y) {
            Ordering::Greater
        } else {
            // ordering is transitive
            x_constraints
                .before
                .iter()
                .find_map(|after_x| {
                    // x < after_x and after_x < y => x < y
                    if self.required_print_ord(after_x, y) == Ordering::Less {
                        Some(Ordering::Less)
                    } else {
                        None
                    }
                })
                .or_else(|| {
                    x_constraints.after.iter().find_map(|before_x| {
                        // before_x < x and y < before_x => y < x <=> x > y
                        if self.required_print_ord(y, before_x) == Ordering::Less {
                            Some(Ordering::Greater)
                        } else {
                            None
                        }
                    })
                });
            Ordering::Equal
        }
    }
}

#[derive(Default, Debug)]
struct Update(Vec<u32>);

impl Update {
    pub fn correct_order(&self, constraints: &UpdateConstraints) -> bool {
        self.0
            .windows(2)
            .all(|p| constraints.required_print_ord(&p[0], &p[1]) != Ordering::Greater)
    }
    pub fn middle(&self) -> PageNumber {
        let Update(pages) = self;

        pages[pages.len() / 2]
    }

    pub fn sort(&mut self, constraints: &UpdateConstraints) {
        let Update(pages) = self;
        pages.sort_by(|x, y| constraints.required_print_ord(x, y))
    }
}

fn parse_constraint(constraint: &str) -> (PageNumber, PageNumber) {
    let (before, after) = constraint.split_once('|').unwrap();
    let before = before.parse().unwrap();
    let after = after.parse().unwrap();

    (before, after)
}
fn parse(input: &str) -> (UpdateConstraints, Vec<Update>) {
    let mut lines = input.lines();

    let mut update_constraints = UpdateConstraints::default();
    let constraints = lines.by_ref().take_while(|l| !l.is_empty());
    constraints.for_each(|constraint| {
        let (before, after) = parse_constraint(constraint);
        update_constraints.constraint(before, after)
    });

    let updates = lines
        .map(|update| {
            Update(
                update
                    .split(',')
                    .map(|page_number| page_number.parse().unwrap())
                    .collect(),
            )
        })
        .collect();

    (update_constraints, updates)
}

pub fn part_one(input: &str) -> Option<u32> {
    let (constraints, updates) = parse(input);

    Some(
        updates
            .into_iter()
            .filter_map(|update| {
                // middle values of correctly sorted
                if update.correct_order(&constraints) {
                    Some(update.middle())
                } else {
                    None
                }
            })
            .sum(),
    )
}

pub fn part_two(input: &str) -> Option<u32> {
    let (constraints, updates) = parse(input);

    Some(
        updates
            .into_iter()
            .filter_map(|mut update| {
                // middle values of initially incorrectly sorted after being sorted
                if !update.correct_order(&constraints) {
                    update.sort(&constraints);
                    Some(update.middle())
                } else {
                    None
                }
            })
            .sum(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(143));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(123));
    }
}
