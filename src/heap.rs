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
    child_index.checked_sub(1).unwrap_or_default() / 2
}

#[inline]
fn left_child_of(parent_index: usize) -> usize {
    parent_index * 2 + 1
}

#[inline]
fn right_child_of(parent_index: usize) -> usize {
    parent_index * 2 + 2
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
        if self.size == self.elements.len() {
            self.elements.push(new_t);
        } else {
            self.elements[self.size] = new_t;
        }
        if self.size > 0 {
            self.shift_up();
        }
        self.size += 1;
    }

    pub fn find_top(&self) -> Option<&T> {
        self.elements.first()
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
        if self.size > 0 {
            self.elements.swap(0, self.size);
            self.shift_down();
        }
        result
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
        let mut current_parent = 0;
        loop {
            let left_child = left_child_of(current_parent);
            let right_child = right_child_of(current_parent);
            if left_child >= self.size {
                return;
            }
            let current_child = if right_child < self.size {
                if self.heap_property_satisfied(left_child, right_child) {
                    left_child
                } else {
                    right_child
                }
            } else {
                left_child
            };
            if self.heap_property_satisfied(current_parent, current_child) {
                return;
            }
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

impl<T> Heap<T> {
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.elements.iter().take(self.size)
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
mod tests {

    use super::*;
    use proptest::prelude::*;
    use std::ops::Range;

    proptest! {
        #[test]
        fn heap_property_should_hold_for_all_parents_and_their_children(heap in any_heap::<i32>(1..1000)) {
            check_heap_property_for_all_parents_and_their_children(&heap);
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
        fn extracting_top_item_should_reduce_its_occurrence_by_one(mut heap in any_heap::<i32>(0..1000)) {
            let top_item = heap.find_top().cloned();
            let initial_count = heap.occurrence_of(top_item.as_ref());
            let expected_count = initial_count.checked_sub(1).unwrap_or_default();

            let extracted_item = heap.extract_top();
            let actual_count = heap.occurrence_of(top_item.as_ref());

            assert_eq!(top_item, extracted_item);
            assert_eq!(expected_count, actual_count);
        }

        #[test]
        fn extracting_top_item_should_keep_heap_property_for_all_parents(mut heap in any_heap::<i32>(0..1000)) {
            while heap.extract_top().is_some() {
                check_heap_property_for_all_parents_and_their_children(&heap);
            }
        }

        #[test]
        fn doing_random_inserts_and_extracts_should_not_break_heap_property(mut heap in any_heap::<i32>(0..1000), ops in any_op_seq(0..100)) {
            for op in ops.iter() {
                match *op {
                    Op::Insert(value) => { heap.insert(value); },
                    Op::ExtractTop => { heap.extract_top(); } ,
                };
            }
            check_heap_property_for_all_parents_and_their_children(&heap);
        }
    }

    fn check_heap_property_for_all_parents_and_their_children<T: Ord>(heap: &Heap<T>) {
        let size = heap.size;
        for parent_index in 0..heap.last_parent_index() {
            let child_indices = [left_child_of(parent_index), right_child_of(parent_index)];
            for child_index in child_indices.iter().cloned().filter(|c| *c < size) {
                assert_eq!(
                    heap.heap_property_satisfied(parent_index, child_index),
                    true
                );
            }
        }
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

    #[derive(Debug)]
    enum Op<T> {
        ExtractTop,
        Insert(T),
    }

    fn any_op_seq<T: Arbitrary + Clone + Ord + Default>(
        size: Range<usize>,
    ) -> impl Strategy<Value = Vec<Op<T>>> {
        proptest::collection::vec(any::<T>(), size).prop_map(|vec| {
            vec.iter()
                .cloned()
                .map(|i| {
                    if i < T::default() {
                        Op::ExtractTop
                    } else {
                        Op::Insert(i)
                    }
                })
                .collect()
        })
    }

    impl<T> Heap<T> {
        #[inline]
        fn last_parent_index(&self) -> usize {
            self.size.checked_sub(1).unwrap_or_default() / 2
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
