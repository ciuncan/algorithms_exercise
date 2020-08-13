use std::fmt::Debug;

pub struct Heap<T> {
    elements: Vec<T>,
    size: usize,
    parent_child_relation: ParentChildRelation,
}

#[derive(Clone, Copy, Debug)]
enum ParentChildRelation {
    Greater,
    Smaller,
}

impl ParentChildRelation {
    fn rel<T: Ord>(&self, parent: &T, child: &T) -> bool {
        match *self {
            ParentChildRelation::Smaller => parent <= child,
            ParentChildRelation::Greater => parent >= child,
        }
    }
}

#[inline]
fn parent_of(child_index: usize) -> usize {
    child_index.wrapping_sub(1) / 2
}

#[inline]
fn left_child_of(parent_index: usize) -> usize {
    parent_index * 2 + 1
}

#[inline]
fn right_child_of(parent_index: usize) -> usize {
    parent_index * 2 + 2
}

#[inline]
fn get_children_of(parent_index: usize, last_child: usize) -> Vec<usize> {
    [left_child_of(parent_index), right_child_of(parent_index)]
        .iter()
        .cloned()
        .filter(|i| *i < last_child)
        .collect()
}

impl<T: Ord> Heap<T> {
    pub fn new_min(capacity: usize) -> Heap<T> {
        Heap::new(capacity, ParentChildRelation::Smaller)
    }

    pub fn new_max(capacity: usize) -> Heap<T> {
        Heap::new(capacity, ParentChildRelation::Greater)
    }

    fn new(capacity: usize, parent_child_relation: ParentChildRelation) -> Self {
        Self {
            elements: Vec::with_capacity(capacity),
            size: 0,
            parent_child_relation,
        }
    }

    pub fn insert_all(&mut self, slice: &[T])
    where
        T: Clone,
    {
        for value in slice {
            self.insert(value.clone());
        }
    }

    pub fn insert(&mut self, new_t: T) {
        self.elements.push(new_t);
        if self.size > 0 {
            self.shift_up();
        }
        self.size += 1;
    }

    pub fn extract_top(&mut self) -> Option<T>
    where
        T: Clone,
    {
        if self.size == 0 {
            return None;
        }
        let result = self.find_top().cloned();
        self.size -= 1;
        self.elements.swap(0, self.size);
        self.shift_down();

        result
    }

    pub fn find_top(&self) -> Option<&T> {
        self.elements.first()
    }

    fn shift_up(&mut self) {
        let mut current_child = self.size;
        let mut current_parent = parent_of(current_child);

        while !self.heap_property_satisfied(current_parent, current_child) {
            self.elements.swap(current_parent, current_child);
            if current_parent == 0 {
                break;
            }
            current_child = current_parent;
            current_parent = parent_of(current_child);
        }
    }

    fn shift_down(&mut self) {
        use crate::traits::Randomizable;

        let mut current_parent = 0;
        while let Some(current_child) = get_children_of(current_parent, self.size)
            .into_iter()
            .filter(|c| !self.heap_property_satisfied(current_parent, *c))
            .collect::<Vec<_>>()
            .get_random()
        {
            self.elements.swap(current_parent, current_child);
            current_parent = current_child;
        }
    }

    #[inline]
    fn heap_property_satisfied(&self, parent_index: usize, child_index: usize) -> bool {
        self.parent_child_relation
            .rel(&self.elements[parent_index], &self.elements[child_index])
    }
}

impl<T: Debug + Sized> Debug for Heap<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.debug_struct("Heap")
            .field("elements", &&self.elements[0..self.size])
            .field("size", &self.size)
            .field("parent_child_relation", &self.parent_child_relation)
            .finish()
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use proptest::prelude::*;
    use std::ops::Range;

    proptest! {
        #[test]
        fn heap_property_should_hold_for_all_parents(heap in any_heap::<i32>(1..1000)) {
            for parent_index in 0..heap.last_parent_index() {
                assert_eq!(heap.heap_property_satisfied(parent_index, left_child_of(parent_index)), true);
                assert_eq!(heap.heap_property_satisfied(parent_index, right_child_of(parent_index)), true);
            }
        }

        #[test]
        fn find_top_should_be_minimum_in_any_min_heap(heap in any_min_heap::<i32>(0..1000)) {
            assert_eq!(heap.find_top(), heap.iter().min());
        }

        #[test]
        fn find_top_should_be_maximum_in_any_max_heap(heap in any_max_heap::<i32>(0..1000)) {
            assert_eq!(heap.find_top(), heap.iter().max());
        }

        #[test]
        fn extracting_top_item_should_reduce_its_occurrence_by_one(mut heap in any_heap::<i8>(0..10)) {
            let top_item = heap.find_top().cloned();
            let initial_count = heap.occurrence_of(top_item.as_ref());
            let expected_count = if initial_count == 0 {
                0
            } else {
                initial_count.wrapping_sub(1)
            };

            let extracted_item = heap.extract_top();
            let actual_count = heap.occurrence_of(top_item.as_ref());

            assert_eq!(top_item, extracted_item);
            assert_eq!(expected_count, actual_count);
        }

    }

    #[test]
    fn test_debug() {
        let mut min_heap = Heap::new_min(5);
        min_heap.insert_all(&[3, 2, 5, 4, 7]);
        assert_eq!(
            format!("{:?}", min_heap),
            "Heap { elements: [2, 3, 5, 4, 7], size: 5, parent_child_relation: Smaller }"
                .to_owned()
        );

        let mut min_heap = Heap::new_max(5);
        min_heap.insert_all(&[3, 2, 5, 4, 7]);
        assert_eq!(
            format!("{:?}", min_heap),
            "Heap { elements: [7, 5, 3, 2, 4], size: 5, parent_child_relation: Greater }"
                .to_owned()
        );
    }

    fn any_heap<T>(size: Range<usize>) -> impl Strategy<Value = Heap<T>>
    where
        T: Arbitrary + Ord + Clone,
    {
        prop_oneof![any_min_heap::<T>(size.clone()), any_max_heap::<T>(size),]
    }

    fn any_min_heap<T>(size: Range<usize>) -> impl Strategy<Value = Heap<T>>
    where
        T: Arbitrary + Ord + Clone,
    {
        any_heap_with_rel(size, ParentChildRelation::Smaller)
    }

    fn any_max_heap<T>(size: Range<usize>) -> impl Strategy<Value = Heap<T>>
    where
        T: Arbitrary + Ord + Clone,
    {
        any_heap_with_rel(size, ParentChildRelation::Greater)
    }

    fn any_heap_with_rel<T>(
        size: Range<usize>,
        relation: ParentChildRelation,
    ) -> impl Strategy<Value = Heap<T>>
    where
        T: Arbitrary + Ord + Clone,
    {
        proptest::collection::vec(any::<T>(), size).prop_map(move |v| {
            let mut min_heap = Heap::new(v.len(), relation);
            min_heap.insert_all(&v);
            min_heap
        })
    }

    impl<T> Heap<T> {
        #[inline]
        fn last_parent_index(&self) -> usize {
            self.size.wrapping_sub(1) / 2
        }

        pub fn iter(&self) -> impl Iterator<Item = &T> {
            self.elements.iter().take(self.size)
        }

        pub fn occurrence_of(&self, item: Option<&T>) -> usize
        where
            T: Eq,
        {
            match item {
                None => 0,
                Some(item) => self.iter().filter(|i| *i == item).count(),
            }
        }
    }
}
