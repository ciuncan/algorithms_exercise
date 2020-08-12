use std::cmp::Ordering;
use std::fmt::Debug;

pub fn binary_search<T>(slice: &[T], value: &T) -> Option<usize>
where
    T: Eq + Ord + Debug,
{
    let mut lo = 0;
    let mut hi = slice.len() - 1;

    while lo <= hi {
        let mid = (lo + hi) >> 1;
        let focused = &slice[mid];

        match focused.cmp(value) {
            Ordering::Equal => return Some(mid),
            Ordering::Less => lo = mid + 1,
            Ordering::Greater => hi = mid.wrapping_sub(1),
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::binary_search;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn find_correct_index_of_an_existing_item((i, v) in index_and_sorted_vec::<i32>(1000)) {
            let needle = v[i];
            assert_eq!(binary_search(&v, &needle), Some(i));
        }

        #[test]
        fn find_the_same_result_as_vec_binary_search(v in sorted_vec::<i32>(1000), t in any::<i32>()) {
            let actual = binary_search(&v, &t);
            let existing = v.binary_search(&t).ok();
            assert_eq!(actual, existing);
        }
    }

    fn index_and_sorted_vec<T>(max_size: usize) -> impl Strategy<Value = (usize, Vec<T>)>
    where
        T: Arbitrary + Clone + Ord,
    {
        (1usize..max_size).prop_flat_map(|size| {
            (0..size)
                .prop_flat_map(move |index| sorted_vec::<T>(size).prop_map(move |vec| (index, vec)))
        })
    }

    fn sorted_vec<T>(size: usize) -> impl Strategy<Value = Vec<T>>
    where
        T: Arbitrary + Clone + Ord,
    {
        proptest::collection::btree_set(any::<T>(), size)
            .prop_map(|set| set.iter().cloned().collect::<Vec<_>>())
    }
}
