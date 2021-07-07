use crate::board::PlayerColor;

const DIRECTIONS: [(i8, i8, i8); 13] = [
    (0,0,1),
    (0,1,-1),
    (0,1,0),
    (0,1,1),
    (1,-1,-1),
    (1,-1,0),
    (1,-1,1),
    (1,0,-1),
    (1,0,0),
    (1,0,1),
    (1,1,-1),
    (1,1,0),
    (1,1,1),
];

pub fn check_winner(board: &[[Vec<PlayerColor>; 4]; 4], last: (i8, i8, i8)) {
    for direction in DIRECTIONS {
        let mut num = 1;
        let mut point = last;
        let color = board[point.0 as usize][point.1 as usize][point.2 as usize];

        loop {
            point.0 += direction.0;
            if point.0 < 0 || point.0 > 3 { break; }
            point.1 += direction.1;
            if point.1 < 0 || point.1 > 3 { break; }
            point.2 += direction.2;

            let rod = &board[point.0 as usize][point.1 as usize];

            if let Some(player) = rod.get(point.2 as usize) {
                if *player == color {
                    num += 1;
                    continue;
                }
            }

            break;
        }

        let mut point = last;

        loop {
            point.0 -= direction.0;
            if point.0 < 0 || point.0 > 3 { break; }
            point.1 -= direction.1;
            if point.1 < 0 || point.1 > 3 { break; }
            point.2 -= direction.2;

            let rod = &board[point.0 as usize][point.1 as usize];

            if let Some(player) = rod.get(point.2 as usize) {
                if *player == color {
                    num += 1;
                    continue;
                }
            }

            break;
        }

        if num >= 4 {
            println!(
                "{} wins!",
                match color {
                    PlayerColor::Red => "Red",
                    PlayerColor::Blue => "Blue",
                },
            );
        }
    }
}