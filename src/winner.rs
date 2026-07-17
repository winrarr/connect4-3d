use crate::board::PlayerColor;

const DIRECTIONS: [(i8, i8, i8); 13] = [
    (0, 0, 1),
    (0, 1, -1),
    (0, 1, 0),
    (0, 1, 1),
    (1, -1, -1),
    (1, -1, 0),
    (1, -1, 1),
    (1, 0, -1),
    (1, 0, 0),
    (1, 0, 1),
    (1, 1, -1),
    (1, 1, 0),
    (1, 1, 1),
];

pub fn check_winner(
    board: &[[Vec<PlayerColor>; 4]; 4],
    last: (usize, usize, usize),
) -> Option<PlayerColor> {
    let color = *board[last.0][last.1].get(last.2)?;

    for direction in DIRECTIONS {
        let mut count = 1;
        count += count_in_direction(board, last, direction, color);
        count += count_in_direction(board, last, negate(direction), color);

        if count >= 4 {
            return Some(color);
        }
    }

    None
}

fn count_in_direction(
    board: &[[Vec<PlayerColor>; 4]; 4],
    start: (usize, usize, usize),
    direction: (i8, i8, i8),
    color: PlayerColor,
) -> usize {
    let mut point = (start.0 as i8, start.1 as i8, start.2 as i8);
    let mut count = 0;

    loop {
        point.0 += direction.0;
        point.1 += direction.1;
        point.2 += direction.2;

        if !(0..4).contains(&point.0) || !(0..4).contains(&point.1) || !(0..4).contains(&point.2) {
            break;
        }

        let Some(&next_color) = board[point.0 as usize][point.1 as usize].get(point.2 as usize)
        else {
            break;
        };

        if next_color != color {
            break;
        }

        count += 1;
    }

    count
}

fn negate(direction: (i8, i8, i8)) -> (i8, i8, i8) {
    (-direction.0, -direction.1, -direction.2)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_board() -> [[Vec<PlayerColor>; 4]; 4] {
        Default::default()
    }

    #[test]
    fn detects_a_vertical_four() {
        let mut board = empty_board();
        board[1][2] = vec![PlayerColor::Red; 4];

        assert_eq!(check_winner(&board, (1, 2, 3)), Some(PlayerColor::Red));
    }

    #[test]
    fn does_not_count_disconnected_pieces() {
        let mut board = empty_board();
        board[0][0] = vec![PlayerColor::Blue, PlayerColor::Blue];
        board[1][0] = vec![PlayerColor::Blue];
        board[2][0] = vec![PlayerColor::Blue];

        assert_eq!(check_winner(&board, (2, 0, 0)), None);
    }
}
