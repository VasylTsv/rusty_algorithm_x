use std::collections::*;
use std::sync::mpsc;

use core::cell::RefCell;

pub type ItemType = u32;
pub type ConditionType = u32;
pub type Problem = HashMap<ItemType, Vec<ConditionType>>;

// The structure used internally by the algorithm has an array (map, actually) of column references (.1) and true for required conditions (columns) 
type RowRef = (bool, RefCell<HashSet<ItemType>>);
type ColumnType = HashMap<ConditionType, RowRef>;
// Scratch has the list of temporarily removed columns and a number of non-optional ones in that list
type Scratch = (i32, Vec<RowRef>);
type Solution = Vec<ItemType>;

// Find the "most constrained" required condition (the one with the lowest number of items attached).
// It is guaranteed to exist at this point - the function is called when there are still required conditions,
// so in the worst case there will an unsatisfiable one. This makes the unwrap() safe to perform.
fn most_constrained(columns: &ColumnType) -> &ItemType {
    columns.iter()
    .filter(|e| e.1.0)
    .min_by_key(|e| e.1.1.borrow().len())
    .unwrap().0
}

// The main solver function. The logic is actually quite simple and not dissimilar to the regular backtracking, except that the select/unselect
// can operate on a large subset of items at once. Scratch structure is used to backtrack.
fn solve(columns: &mut ColumnType, rows: &Problem, solution: &mut Solution, required: i32, sender: &mpsc::Sender<Solution>) {
    // All required conditions are satisfied, yield the current solution candidate as a good solution
    if required == 0 {
        sender.send(solution.clone()).unwrap();
    } else {
        // Note that this has to be done on a line separate from the loop below otherwise columns will remain borrowed for the loop body duration.
        // The clone() is required by the algorithm as that particular column may be modified in inner calls.
        let order = columns.get(most_constrained(&columns)).unwrap().1.borrow().clone();
        for item in order {
            solution.push(item);
            let scratch = select(columns, rows, item);
            if !scratch.1.is_empty() {
                solve(columns, rows, solution, required - scratch.0, sender);
                deselect(columns, rows, item, scratch);
            }
            solution.pop();
        }
    }
}

fn select(columns: &mut ColumnType, rows: &Problem, item: ItemType) -> Scratch {
    let mut scratch = (0, Vec::<RowRef>::new());
    if let Some(yr) = rows.get(&item) {
        for j in yr {
            // This line (and a similar line in deselect) is the reason from the RefCell use. Without it the mutable borrow inside
            // the loop is impossible
            if let Some(xj) = columns.get(&j) {
                for i in xj.1.borrow().iter() {
                    if let Some(yi) = rows.get(&i) {
                        for k in yi {
                            if k != j {
                                if let Some(xk) = columns.get(&k) {
                                    xk.1.borrow_mut().remove(&i);
                                }
                            }
                        }
                    }
                }
                if xj.0 {
                    scratch.0 += 1;
                }
                scratch.1.push(columns.remove(&j).unwrap());
            }
        }
    }
    scratch
}

fn deselect(columns: &mut ColumnType, rows: &Problem, item: ItemType, mut scratch: Scratch) {
    if let Some(yr) = rows.get(&item) {
        for j in yr.iter().rev() {
            if let Some(xj) = scratch.1.pop() {
                if xj.0 {
                    scratch.0 -= 1;
                }
                for i in xj.1.borrow().iter() {
                    if let Some(yi) = rows.get(&i) {
                        for k in yi {
                            if k != j {
                                if let Some(xk) = columns.get(&k) {
                                    xk.1.borrow_mut().insert(*i);
                                }
                            }
                        }
                    }
                }
                columns.insert(*j, xj);
            }
        }
    }
}

fn preselect(columns: &mut ColumnType, rows: &Problem, solution: &mut Solution, required: i32, item: ItemType) -> i32 {
    let scratch = select(columns, rows, item);
    solution.push(item);
    required - scratch.0
}

// External API to the algorithm. Transforms the problem representation into the form required by the algorithm
pub fn run(rows: &Problem, preselected: Option<&Vec<ItemType>>, optional: Option<&Vec<ConditionType>>, sender: &mpsc::Sender<Solution>) {
    // Collect all conditions references by the problem
    let mut conditions = HashSet::<ConditionType>::new();
    for v in rows {
        for c in v.1 {
            conditions.insert(*c);
        }
    }

    // Problem validation
    // a) All optional conditions actually exist in the problem
    // b) There are no items with only optional conditions (this makes algorithm inconsistent, can be fixed by special case)
    if let Some(optional) = optional {
        for c in optional {
            assert!(conditions.contains(c));
        }

        for item in rows {
            assert!(item.1.iter().any(|e| !optional.contains(e)));
        }
    }
    // c) All preselected items actually exist in the problem
    if let Some(preselected) = preselected {
        for i in preselected {
            assert!(rows.contains_key(i));
        }
    }

    // The problem represents only row links. Transform it into column links representation (the algorithm
    // requires both but only modifies the columnar one)
    let mut columns = ColumnType::new();
    let mut required = conditions.len() as i32;
    for j in conditions {
        let v:  HashSet<ItemType> = rows.iter().filter(|e| e.1.contains(&j)).map(|e| *e.0).collect();

        let reqflag =
            if let Some(optional) = optional {
                optional.contains(&j) == false
            } else {
                true
            };

        if reqflag == false {
            required -= 1;
        }

        columns.insert(j, (reqflag, RefCell::new(v)));
    }

    let mut sol = Solution::new();

    // Last step before the algorithm, preselect the necessary items (this effectively removes them
    // from the algorithm lookups and tags the conditions properly)
    if let Some(psel) = preselected {
        for s in psel {
            required = preselect(&mut columns, rows, &mut sol, required, *s);
        }
    }

    // Run the algorithm
    solve(&mut columns, &rows, &mut sol, required, &sender);
}

// Helper function for cases when it is preferrable to build the problem data item by item
pub fn set_condition(problem: &mut Problem, item:ItemType, condition: ConditionType)
{
    if let Some(row) = problem.get_mut(&item) {
        row.push(condition);
    } else {
        problem.insert(item, vec![condition]);    
    }
}

// Helper wrapper allowing to hide the thread setup
// This setup is a syntactical approximation of yielding - instead of true yield the return values will be sent
// over sender/receiver. It is obviously not the same as yield but it may actually be better solution in some
// cases, like this algorithm: we don't care for the yielded value to actually float through the caller chain, and
// it allows processing of a solution while looking for another solution. 
// Note that the call will steal the first parameter to pass it to the worker thread. In most cases this is completely
// acceptable. If not, the wrapper needs to be called on the clone.
macro_rules! solutions {
    ($columns:expr, $preselected:expr, $optional:expr) => ({
        let (sender, receiver) = mpsc::channel();
        let _handle = thread::spawn(move || {
            algorithmx::run(&$columns, $preselected, $optional, &sender);
        });
    
        receiver
    });
}
