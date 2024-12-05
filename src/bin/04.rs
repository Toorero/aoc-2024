advent_of_code::solution!(4);

type Quizz = Vec<Vec<char>>;
type Coord = (usize, usize);

fn parse(input: &str) -> Quizz {
    input.lines().map(|line| line.chars().collect()).collect()
}

const XMAS_STENCILS: [[Coord; 4]; 4] = [
    // normal
    [(0, 0), (0, 1), (0, 2), (0, 3)],
    // vertical
    [(0, 0), (1, 0), (2, 0), (3, 0)],
    // diagonal(\)
    [(0, 0), (1, 1), (2, 2), (3, 3)],
    // diagonal(/)
    [(0, 3), (1, 2), (2, 1), (3, 0)],
];
const XMAS: [[char; 4]; 2] = [['X', 'M', 'A', 'S'], ['S', 'A', 'M', 'X']];

const X_MAS_STENCIL: [[Coord; 3]; 2] = [[(0, 0), (1, 1), (2, 2)], [(2, 0), (1, 1), (0, 2)]];
const X_MAS: [[char; 3]; 2] = [['M', 'A', 'S'], ['S', 'A', 'M']];

fn matches(quizz: &Quizz, coord: Coord) -> usize {
    let (x, y) = coord;

    XMAS_STENCILS
        .map(|stencil| {
            let word = stencil.map(|(dx, dy)| {
                *quizz
                    .get(x + dx)
                    .and_then(|l| l.get(y + dy))
                    .unwrap_or(&' ')
            });
            XMAS.iter().any(|xmas| xmas == &word)
        })
        .into_iter()
        .filter(|b| b == &true)
        .count()
}

fn x_matches(quizz: &Quizz, coord: Coord) -> bool {
    let (x, y) = coord;

    X_MAS_STENCIL
        .map(|stencil| {
            let word = stencil.map(|(dx, dy)| {
                *quizz
                    .get(x + dx)
                    .and_then(|l| l.get(y + dy))
                    .unwrap_or(&' ')
            });
            X_MAS.iter().any(|xmas| xmas == &word)
        })
        .into_iter()
        .all(|b| b)
}

pub fn part_one(input: &str) -> Option<usize> {
    let chars = parse(input);

    let mut xmas_count = 0;
    for x in 0..chars.len() {
        for y in 0..chars[0].len() {
            let coord = (x, y);
            xmas_count += matches(&chars, coord);
        }
    }
    Some(xmas_count)
}

pub fn part_two(input: &str) -> Option<usize> {
    let chars = parse(input);

    let mut xmas_count = 0;
    for x in 0..chars.len() {
        for y in 0..chars[0].len() {
            let coord = (x, y);
            if x_matches(&chars, coord) {
                xmas_count += 1;
            }
        }
    }
    Some(xmas_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(18));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(9));
    }
}
