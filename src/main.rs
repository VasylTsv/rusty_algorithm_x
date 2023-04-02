#[macro_use]
mod algorithmx;
use std::sync::mpsc;
use std::thread;


fn main() {
    println!("Queens");
    queens();
    println!("Sudoku");
    sudoku();
    println!("Pentomino");
    pentomino();    
}

// Test problems ported from older C++ program. Most of the code was ported 1:1 which resulted in rather ugly and
// not particularly idiomatic code. To be cleaned up

// 8 Queens problem solver. Cover with some optional conditions
fn queens() {
    const NUMBER_OF_QUEENS: u32 = 11;

    let mut problem = algorithmx::Problem::new();

    for row in 0..NUMBER_OF_QUEENS {
        for col in 0..NUMBER_OF_QUEENS {
            let r = col * NUMBER_OF_QUEENS + row;
            algorithmx::set_condition(&mut problem, r, row);
            algorithmx::set_condition(&mut problem, r, col+NUMBER_OF_QUEENS);
            algorithmx::set_condition(&mut problem, r, col+row + 2*NUMBER_OF_QUEENS);
            algorithmx::set_condition(&mut problem, r, col + 5*NUMBER_OF_QUEENS - row);
        }
    }

    let mut optional = Vec::<u32>::new();
    optional.extend(2*NUMBER_OF_QUEENS..4*NUMBER_OF_QUEENS-1);
    optional.extend(4*NUMBER_OF_QUEENS+1..6*NUMBER_OF_QUEENS);


    let mut counter = 1;
    for solution in solutions!(problem, None, Some(&optional)) {
        
        println!("Solution {}", counter);
        counter += 1;

        let mut sol = [0; NUMBER_OF_QUEENS as usize];

        for x in solution {
            sol[(x % NUMBER_OF_QUEENS) as usize] = x / NUMBER_OF_QUEENS;
        }

        for x in sol {
            for i in 0..NUMBER_OF_QUEENS {
                print!("{}", if i == x { 'X' } else { '.' } );
            }
            println!();
        }
    }
}


// Sudoku solver. Exact cover problem with preselected items
fn sudoku() {
    let mut problem = algorithmx::Problem::new();

    // Each item is a combo of vertical position (row), horizontal position (column), and digit, encoded into one "element".
    // Hence there are four conditions for each item -
    // a) no other item takes the same spot
    // b) no other item is on the same row
    // c) no other item is on the same column
    // d) no other item is in the same 3x3 square
    // All conditions are also encoded by packing into one value
    const CELL_START:u32 = 0;
    const ROW_START:u32 = 81;
    const COL_START:u32 = 162;
    const SQUARE_START:u32 = 243;

    for r in 0..9 {
        for c in 0..9 {
            for n in 0..9 {
                let element = r*81+c*9+n;
                let sq = (r/3)*3 + (c/3);
                algorithmx::set_condition(&mut problem, element, CELL_START + 9 * r + c);
                algorithmx::set_condition(&mut problem, element, ROW_START + 9 * r + n);
                algorithmx::set_condition(&mut problem, element, COL_START + 9 * c + n);
                algorithmx::set_condition(&mut problem, element, SQUARE_START + 9 * sq + n);
            }
        }
    }

    let mut presel = Vec::<algorithmx::ItemType>::new();

    // Helper for more obvious encoding of the actual puzzle
    macro_rules! set {
        ($r:expr, $c:expr, $n:expr) => {
            presel.push($r*81+$c*9+$n-1);
        };
    }

    // The test puzzle
    set!(0, 0, 5); set!(0, 1, 3); set!(0, 4, 7);
    set!(1, 0, 6); set!(1, 3, 1); set!(1, 4, 9); set!(1, 5, 5);
    set!(2, 1, 9); set!(2, 2, 8); set!(2, 7, 6);
    set!(3, 0, 8); set!(3, 4, 6); set!(3, 8, 3);
    set!(4, 0, 4); set!(4, 3, 8); set!(4, 5, 3); set!(4, 8, 1);
    set!(5, 0, 7); set!(5, 4, 2); set!(5, 8, 6);
    set!(6, 1, 6); set!(6, 6, 2); set!(6, 7, 8);
    set!(7, 3, 4); set!(7, 4, 1); set!(7, 5, 9); set!(7, 8, 5);
    set!(8, 4, 8); set!(8, 7, 7); set!(8, 8, 9);

    for solution in solutions!(problem, Some(&presel), None) {
        // The solution is just an unsorted array of items. As the encoding uses higher values for rows and columns,
        // simple sorting makes it much easier to work with
        let mut solution = solution;
        solution.sort();
        // And just some pretty output
        let mut ctr = 0;
        for x in solution {
            print!(" {}", x%9+1);
            ctr += 1;
            if ctr == 27 || ctr == 54 {
                println!("\n-------+-------+-------")
            } else if ctr % 9 == 0 {
                println!();
            } else if ctr % 3 == 0 {
                print!(" |");
            }
        }
    }
}

// Pentomino 6x10. No preselected items or optional conditions
fn pentomino()
{
    struct PieceInfo
    {
        t: char,
        coverage: [u8;5],
    }

    let piece_info =
    [
        PieceInfo { t:'F', coverage:[0x18, 0x30, 0x10, 0x00, 0x00] },
        PieceInfo { t:'F', coverage:[0x08, 0x0e, 0x04, 0x00, 0x00] },
        PieceInfo { t:'F', coverage:[0x08, 0x0c, 0x18, 0x00, 0x00] },
        PieceInfo { t:'F', coverage:[0x08, 0x1c, 0x04, 0x00, 0x00] },
        PieceInfo { t:'F', coverage:[0x18, 0x0c, 0x08, 0x00, 0x00] },
        PieceInfo { t:'F', coverage:[0x08, 0x38, 0x10, 0x00, 0x00] },
        PieceInfo { t:'F', coverage:[0x08, 0x18, 0x0c, 0x00, 0x00] },
        PieceInfo { t:'F', coverage:[0x08, 0x1c, 0x10, 0x00, 0x00] },
        PieceInfo { t:'I', coverage:[0xf8, 0x00, 0x00, 0x00, 0x00] },
        PieceInfo { t:'I', coverage:[0x08, 0x08, 0x08, 0x08, 0x08] },
        PieceInfo { t:'L', coverage:[0x78, 0x40, 0x00, 0x00, 0x00] },
        PieceInfo { t:'L', coverage:[0x78, 0x08, 0x00, 0x00, 0x00] },
        PieceInfo { t:'L', coverage:[0x08, 0x08, 0x08, 0x18, 0x00] },
        PieceInfo { t:'L', coverage:[0x08, 0x08, 0x08, 0x0c, 0x00] },
        PieceInfo { t:'L', coverage:[0x08, 0x78, 0x00, 0x00, 0x00] },
        PieceInfo { t:'L', coverage:[0x08, 0x0f, 0x00, 0x00, 0x00] },
        PieceInfo { t:'L', coverage:[0x18, 0x10, 0x10, 0x10, 0x00] },
        PieceInfo { t:'L', coverage:[0x18, 0x08, 0x08, 0x08, 0x00] },
        PieceInfo { t:'P', coverage:[0x18, 0x18, 0x08, 0x00, 0x00] },
        PieceInfo { t:'P', coverage:[0x18, 0x18, 0x10, 0x00, 0x00] },
        PieceInfo { t:'P', coverage:[0x08, 0x18, 0x18, 0x00, 0x00] },
        PieceInfo { t:'P', coverage:[0x08, 0x0c, 0x0c, 0x00, 0x00] },
        PieceInfo { t:'P', coverage:[0x38, 0x18, 0x00, 0x00, 0x00] },
        PieceInfo { t:'P', coverage:[0x38, 0x30, 0x00, 0x00, 0x00] },
        PieceInfo { t:'P', coverage:[0x18, 0x1c, 0x00, 0x00, 0x00] },
        PieceInfo { t:'P', coverage:[0x18, 0x38, 0x00, 0x00, 0x00] },
        PieceInfo { t:'N', coverage:[0x18, 0x70, 0x00, 0x00, 0x00] },
        PieceInfo { t:'N', coverage:[0x18, 0x0e, 0x00, 0x00, 0x00] },
        PieceInfo { t:'N', coverage:[0x38, 0x60, 0x00, 0x00, 0x00] },
        PieceInfo { t:'N', coverage:[0x38, 0x0c, 0x00, 0x00, 0x00] },
        PieceInfo { t:'N', coverage:[0x08, 0x18, 0x10, 0x10, 0x00] },
        PieceInfo { t:'N', coverage:[0x08, 0x0c, 0x04, 0x04, 0x00] },
        PieceInfo { t:'N', coverage:[0x08, 0x08, 0x18, 0x10, 0x00] },
        PieceInfo { t:'N', coverage:[0x08, 0x08, 0x0c, 0x04, 0x00] },
        PieceInfo { t:'T', coverage:[0x38, 0x10, 0x10, 0x00, 0x00] },
        PieceInfo { t:'T', coverage:[0x08, 0x08, 0x1c, 0x00, 0x00] },
        PieceInfo { t:'T', coverage:[0x08, 0x0e, 0x08, 0x00, 0x00] },
        PieceInfo { t:'T', coverage:[0x08, 0x38, 0x08, 0x00, 0x00] },
        PieceInfo { t:'U', coverage:[0x28, 0x38, 0x00, 0x00, 0x00] },
        PieceInfo { t:'U', coverage:[0x38, 0x28, 0x00, 0x00, 0x00] },
        PieceInfo { t:'U', coverage:[0x18, 0x08, 0x18, 0x00, 0x00] },
        PieceInfo { t:'U', coverage:[0x18, 0x10, 0x18, 0x00, 0x00] },
        PieceInfo { t:'V', coverage:[0x38, 0x08, 0x08, 0x00, 0x00] },
        PieceInfo { t:'V', coverage:[0x38, 0x20, 0x20, 0x00, 0x00] },
        PieceInfo { t:'V', coverage:[0x08, 0x08, 0x0e, 0x00, 0x00] },
        PieceInfo { t:'V', coverage:[0x08, 0x08, 0x38, 0x00, 0x00] },
        PieceInfo { t:'W', coverage:[0x18, 0x30, 0x20, 0x00, 0x00] },
        PieceInfo { t:'W', coverage:[0x18, 0x0c, 0x04, 0x00, 0x00] },
        PieceInfo { t:'W', coverage:[0x08, 0x18, 0x30, 0x00, 0x00] },
        PieceInfo { t:'W', coverage:[0x08, 0x0c, 0x06, 0x00, 0x00] },
        PieceInfo { t:'X', coverage:[0x08, 0x1c, 0x08, 0x00, 0x00] },
        PieceInfo { t:'Y', coverage:[0x78, 0x10, 0x00, 0x00, 0x00] },
        PieceInfo { t:'Y', coverage:[0x78, 0x20, 0x00, 0x00, 0x00] },
        PieceInfo { t:'Y', coverage:[0x08, 0x3c, 0x00, 0x00, 0x00] },
        PieceInfo { t:'Y', coverage:[0x08, 0x1e, 0x00, 0x00, 0x00] },
        PieceInfo { t:'Y', coverage:[0x08, 0x18, 0x08, 0x08, 0x00] },
        PieceInfo { t:'Y', coverage:[0x08, 0x0c, 0x08, 0x08, 0x00] },
        PieceInfo { t:'Y', coverage:[0x08, 0x08, 0x18, 0x08, 0x00] },
        PieceInfo { t:'Y', coverage:[0x08, 0x08, 0x0c, 0x08, 0x00] },
        PieceInfo { t:'Z', coverage:[0x18, 0x10, 0x30, 0x00, 0x00] },
        PieceInfo { t:'Z', coverage:[0x18, 0x08, 0x0c, 0x00, 0x00] },
        PieceInfo { t:'Z', coverage:[0x08, 0x38, 0x20, 0x00, 0x00] },
        PieceInfo { t:'Z', coverage:[0x08, 0x0e, 0x02, 0x00, 0x00] },
    ];

    let mut problem = algorithmx::Problem::new();

    // The puzzle is traditional - 6x10 rectangle
    // The code below can be fairly easily modified to solve any puzzle: just use field larger than 6x10, large
    // enough to cover the entire puzzle, and check each piece against the puzzle cells instead of the field
    // boundaries like below.
    let mut index: u32 = 0;
    for piece in &piece_info {
        // Convert 'coverage' fields into offsets. x can actually be negative but y is always positive.
        #[derive(Clone,Copy)]
        struct Point {
            x: i32,
            y: i32,
        }
        impl Point {
            fn new() ->Self { Point{x:0,y:0} }
        }

        let mut offset = [Point::new(); 5];
        let mut count: usize = 0;
        for r in 0..5 {
            for off in 0..8 {
                if (piece.coverage[r] & (1<<off)) != 0 {
                    offset[count].y = r as i32;
                    offset[count].x = off - 3;
                    count += 1;
                }
            }
        }

        // Go through all 60 cells
        for x in 0..10 {
            for y in 0..6 {
                // Does this piece fit if placed on this cell?
                let mut good: bool = true;
                for off in 0..5 {
                    if x + offset[off].x >= 10 || x + offset[off].x < 0 || y + offset[off].y >= 6 {
                        good = false;
                        break;
                    }
                }

                // If it does, add constraints for five cells that this piece covers and the piece type so
                // each type will only be used once
                let piece_here = index * 60 + (y * 10 + x) as u32;
                if good {
                    for off in 0..5 {
                        algorithmx::set_condition(&mut problem, piece_here, ((x + offset[off].x) * 10 + y + offset[off].y) as u32);
                    }

                    algorithmx::set_condition(&mut problem, piece_here, 4000 + piece.t as u32);
                }
            }
        }
        index += 1;
    }

    let mut counter: u32 = 1;
    for solution in solutions!(problem, None, None) {
        let mut sol = [[' '; 10];6];
        for x in solution {
            for r in 0..5 {
                for off in 0..8 {
                    if (piece_info[(x/60) as usize].coverage[r] & (1<<off)) != 0 {
                        sol[((x / 10) % 6 + r as u32) as usize][(x % 10 + off - 3) as usize] = piece_info[(x / 60) as usize].t
                    }
                }
            }
            sol[((x / 10) % 6) as usize][(x % 10) as usize] = piece_info[(x / 60) as usize].t;
        }

        println!("Solution {}\n", counter);
        for i in 0..6 {
            let s: String = sol[i].into_iter().collect();
            println!("{}", s);
        }
        println!();
        counter += 1;
    }
}