use regex::Regex;

advent_of_code::solution!(3);

#[non_exhaustive]
#[derive(Debug)]
pub enum Instruction {
    Mul(u32, u32),
    Do(),
    Dont(),
}

fn parse(memory: &str) -> Vec<Instruction> {
    let re = Regex::new(r"(?:(mul)\(([0-9]+),([0-9]+)\))|(?:(do|don't)\(\))").unwrap();

    re.captures_iter(memory)
        .filter_map(|c| {
            let op = c.get(1).or_else(|| c.get(4)).map(|m| m.as_str()).unwrap();

            match op {
                "mul" => {
                    let x = c[2].parse().unwrap();
                    let y = c[3].parse().unwrap();
                    Some(Instruction::Mul(x, y))
                }
                "do" => Some(Instruction::Do()),
                "don't" => Some(Instruction::Dont()),
                _ => None,
            }
        })
        .collect()
}

pub fn part_one(input: &str) -> Option<u32> {
    let instructions = parse(input);

    Some(instructions.into_iter().fold(0, |acc, ins| {
        acc + if let Instruction::Mul(x, y) = ins {
            x * y
        } else {
            0
        }
    }))
}

pub fn part_two(input: &str) -> Option<u32> {
    let instructions = parse(input);

    Some(
        instructions
            .into_iter()
            .fold((0, true), |(acc, mult_enabled), ins| {
                match (mult_enabled, ins) {
                    (true, Instruction::Mul(x, y)) => (acc + x * y, mult_enabled),
                    (false, Instruction::Mul(..)) => (acc, mult_enabled),
                    (_, Instruction::Do()) => (acc, true),
                    (_, Instruction::Dont()) => (acc, false),
                }
            })
            .0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(161));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(48));
    }
}
