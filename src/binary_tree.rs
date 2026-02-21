use core::{
    cmp::Ordering,
    fmt,
    ops::{Index, IndexMut},
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum AVLTreeError {
    Overflow,
}

pub(super) struct KeyValuePair<K, V> {
    key: K,
    value: V,
}

impl<K, V> KeyValuePair<K, V> where K: Ord + Eq {
    pub(super) const fn new(key: K, value: V) -> Self {
        Self {
            key,
            value
        }
    }
}

impl<K, V> PartialEq for KeyValuePair<K, V>
where
    K: Ord + Eq,
{
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<K, V> Eq for KeyValuePair<K, V> where K: Ord + Eq {}

impl<K, V> PartialOrd for KeyValuePair<K, V>
where
    K: Ord + Eq,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.key.cmp(&other.key))
    }
}

impl<K, V> Ord for KeyValuePair<K, V>
where
    K: Ord + Eq,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}

impl<K, V> fmt::Debug for KeyValuePair<K, V>
where
    K: fmt::Debug + Ord,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?}, {:?})", self.key, self.value)
    }
}

struct DoubleList<T, const CAPACITY: usize>([[T; CAPACITY]; 2]);

impl<T, const CAPACITY: usize> DoubleList<T, CAPACITY> {
    pub(crate) const fn new(inner: [[T; CAPACITY]; 2]) -> Self {
        Self(inner)
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0[0].iter().chain(self.0[1].iter())
    }

    pub fn iter_reverse(&self) -> impl Iterator<Item = &T> {
        self.0[1].iter().rev().chain(self.0[0].iter().rev())
    }

    pub fn len(&self) -> usize {
        CAPACITY * 2
    }
}

impl<T, const CAPACITY: usize> Index<usize> for DoubleList<T, CAPACITY> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        let outer_index = index / CAPACITY;
        let inner_index = index % CAPACITY;
        &self.0[outer_index][inner_index]
    }
}

impl<T, const CAPACITY: usize> IndexMut<usize> for DoubleList<T, CAPACITY> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let outer_index = index / CAPACITY;
        let inner_index = index % CAPACITY;
        &mut self.0[outer_index][inner_index]
    }
}

impl<T: fmt::Debug, const CAPACITY: usize> fmt::Debug for DoubleList<T, CAPACITY>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for i in 0..CAPACITY {
            write!(f, "{:?}, ", self.0[0][i])?;
        }

        for i in 0..CAPACITY - 1 {
            write!(f, "{:?}, ", self.0[1][i])?;
        }

        write!(f, "{:?}]", self.0[1][CAPACITY - 1])?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BalanceFactor {
    LeftHeavy,
    Equal,
    RightHeavy,
}

#[allow(dead_code)]
pub(crate) struct AVLTreeMap<K: Ord, V, const CAPACITY: usize>(
    DoubleList<Option<KeyValuePair<K, V>>, CAPACITY>,
);

#[allow(dead_code)]
impl<K: Ord, V, const CAPACITY: usize> AVLTreeMap<K, V, CAPACITY>
where
    K: Ord + Eq,
{
    pub(crate) const fn new() -> Self {
        Self(DoubleList::new([const { [const { None }; CAPACITY] }; 2]))
    }

    // Simple log2
    fn log2(&self, value: usize) -> usize {
        let mut value = value;

        let mut log2: usize = 0;
        while value > 0 {
            value >>= 1;
            log2 += 1;
        }

        log2
    }

    pub(crate) fn get_height(&self) -> usize {
        let length = match self.0.iter_reverse().position(|x| x.is_some()) {
            Some(length) => self.0.len() - length,
            None => 0,
        };

        self.log2(length)
    }

    pub(super) fn get_balance_factor(&self, index: usize) -> BalanceFactor {
        let left_height = {
            let mut index = index;
            let mut height: usize = 0;
            while let Some(_) = self.get_left_child(index) {
                index = Self::get_left_child_index(index);
                height += 1;
            }
            height
        };

        let right_height = {
            let mut index = index;
            let mut height: usize = 0;
            while let Some(_) = self.get_right_child(index) {
                index = Self::get_right_child_index(index);
                height += 1;
            }
            height
        };

        if left_height == right_height {
            BalanceFactor::Equal
        } else if left_height > right_height {
            BalanceFactor::LeftHeavy
        } else {
            BalanceFactor::RightHeavy
        }
    }

    pub(super) fn right_rotate(&mut self, index: usize) {
        let root_index = index;
        let left_child_index = Self::get_left_child_index(root_index);
        let right_child_index = Self::get_left_child_index(root_index);
    }

    pub(crate) fn push(&mut self, key: K, value: V) -> Result<(), AVLTreeError> {
        // Start as a normal binary tree. Follow the path down and choose child node based on the
        // ordering
        let element = KeyValuePair::new(key, value);
        let mut index: usize = 0;
        loop {
            if index > self.0.len() {
                return Err(AVLTreeError::Overflow);
            }
            if let Some(compared_element) = &self.0[index] {
                if element == *compared_element {
                    return Ok(());
                } else if element < *compared_element {
                    index = Self::get_left_child_index(index);
                } else {
                    index = Self::get_right_child_index(index);
                }
            } else {
                let _ = self.0[index].insert(element);
                return Ok(());
            }
        }
    }

    fn get_parent_node(index: usize) -> usize {
        if index > 0 {
            (index - 1) / 2
        } else {
            // Root node
            0
        }
    }

    fn get_left_child_index(index: usize) -> usize {
        index * 2 + 1
    }

    fn get_right_child_index(index: usize) -> usize {
        index * 2 + 2
    }

    fn get_left_child(&self, index: usize) -> Option<&KeyValuePair<K, V>> {
        let index = Self::get_left_child_index(index);
        if index < self.0.len() {
            self.0[index].as_ref()
        } else {
            None
        }
    }

    fn get_right_child(&self, index: usize) -> Option<&KeyValuePair<K, V>> {
        let index = Self::get_right_child_index(index);
        if index < self.0.len() {
            self.0[index].as_ref()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::binary_tree::{AVLTreeMap, BalanceFactor, DoubleList, KeyValuePair};

    type MyDoubleList = DoubleList<usize, 3>;
    type MyTree = AVLTreeMap<usize, (), 10>;

    fn get_double_liet() -> MyDoubleList {
        MyDoubleList::new([[0; 3]; 2])
    }

    #[test]
    fn create_double_list() {
        let _ = get_double_liet();
    }

    #[test]
    fn index_double_list() {
        let mut list = get_double_liet();
        list[0] = 1;
        list[2] = 2;
        list[3] = 3;
        list[5] = 4;

        assert_eq!(1, list.0[0][0]);
        assert_eq!(2, list.0[0][2]);
        assert_eq!(3, list.0[1][0]);
        assert_eq!(4, list.0[1][2]);
    }

    #[test]
    fn iterate_double_list() {
        let mut list = get_double_liet();
        list[0] = 1;
        list[2] = 2;
        list[3] = 3;
        list[5] = 4;

        const RESULT: [usize; 6] = [1, 0, 2, 3, 0, 4];
        let mut index = 0;

        for element in list.iter() {
            assert_eq!(RESULT[index], *element);
            index += 1;
        }
    }

    #[test]
    fn reverse_iterate_double_list() {
        let mut list = get_double_liet();
        list[0] = 1;
        list[2] = 2;
        list[3] = 3;
        list[5] = 4;

        const RESULT: [usize; 6] = [4, 0, 3, 2, 0, 1];
        let mut index = 0;

        for element in list.iter_reverse() {
            assert_eq!(RESULT[index], *element);
            index += 1;
        }
    }

    #[test]
    fn create_treee() {
        let _ = MyTree::new();
    }

    #[test]
    fn push_element() {
        let mut tree = MyTree::new();
        tree.push(0, ()).unwrap();
    }

    #[test]
    fn get_height() {
        let mut tree = MyTree::new();
        tree.push(2, ()).unwrap();
        tree.push(1, ()).unwrap();
        tree.push(3, ()).unwrap();

        println!("Tree: {:?}", tree.0);

        assert_eq!(tree.get_height(), 2);

        tree.push(0, ()).unwrap();
        assert_eq!(tree.get_height(), 3);
    }

    #[test]
    fn get_equal_balance_factor() {
        //     1
        //    / \
        //  0    2
        let mut tree = MyTree::new();
        tree.push(1, ()).unwrap();
        tree.push(0, ()).unwrap();
        tree.push(2, ()).unwrap();

        println!("Tree: {:?}", tree.0);

        let balance_factor = tree.get_balance_factor(0);
        assert_eq!(balance_factor, BalanceFactor::Equal);
    }

    #[test]
    fn get_left_heavy_balance_factor() {
        //       2
        //      / \
        //    1    3
        //   /
        //  0
        let mut tree = MyTree::new();
        tree.push(2, ()).unwrap();
        tree.push(1, ()).unwrap();
        tree.push(0, ()).unwrap();
        tree.push(3, ()).unwrap();

        println!("Tree: {:?}", tree.0);

        let balance_factor = tree.get_balance_factor(0);
        assert_eq!(balance_factor, BalanceFactor::LeftHeavy);
    }

    #[test]
    fn get_right_heavy_balance_factor() {
        //     1
        //    / \
        //  0    2
        //        \
        //         3
        let mut tree = MyTree::new();
        tree.push(1, ()).unwrap();
        tree.push(0, ()).unwrap();
        tree.push(2, ()).unwrap();
        tree.push(3, ()).unwrap();

        println!("Tree: {:?}", tree.0);

        let balance_factor = tree.get_balance_factor(0);
        assert_eq!(balance_factor, BalanceFactor::RightHeavy);
    }

    #[test]
    fn right_rotation() {
        //       4
        //      / \
        //    2    5
        //   / \
        //  1   3
        let mut tree = MyTree::new();

        let root_index = 0;

        let left_child_index = MyTree::get_left_child_index(root_index);
        let right_child_index = MyTree::get_right_child_index(root_index);

        let left_child_left_child_index = MyTree::get_left_child_index(left_child_index);
        let left_child_right_child_index = MyTree::get_right_child_index(left_child_index);

        let _ = tree.0[root_index].insert(KeyValuePair::new(4, ()));

        let _ = tree.0[left_child_index].insert(KeyValuePair::new(2, ()));
        let _ = tree.0[right_child_index].insert(KeyValuePair::new(5, ()));

        let _ = tree.0[left_child_left_child_index].insert(KeyValuePair::new(1, ()));
        let _ = tree.0[left_child_right_child_index].insert(KeyValuePair::new(3, ()));

        println!("Tree: {:?}", tree.0);
    }
}
