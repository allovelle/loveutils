use std::{collections::Bound, ops::RangeBounds};

#[derive(Debug)]
struct Love(f32);

fn main()
{
    let items = vec![Love(1.1), Love(2.2), Love(3.3), Love(4.4), Love(5.5)];
    let range = 2 ..;
    print_slice(&items[..], range);
}

fn print_slice<T: std::fmt::Debug, U: RangeBounds<usize>>(
    slice: &[T],
    bounds: U,
)
{
    let start = bounds.start_bound();
    let start_index = match start
    {
        Bound::Included(at) => *at,
        Bound::Excluded(_) => todo!(),
        Bound::Unbounded => 0,
    };

    let end = bounds.end_bound();
    let end_index = match end
    {
        Bound::Included(at) | Bound::Excluded(at) => *at,
        Bound::Unbounded => slice.len(),
    };

    for i in start_index .. end_index
    {
        println!("{:?}", slice[i]);
    }
}
