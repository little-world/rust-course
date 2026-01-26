// Pattern 7: Iterator Trait Family
// The core iterator traits (simplified):
// trait IntoIterator {
//     type Item;
//     type IntoIter: Iterator<Item = Self::Item>;
//     fn into_iter(self) -> Self::IntoIter;
// }
//
// trait Iterator {
//     type Item;
//     fn next(&mut self) -> Option<Self::Item>;
// }

// For Vec<T>, three implementations:
// impl IntoIterator for Vec<T>       -> Item = T
// impl IntoIterator for &Vec<T>      -> Item = &T
// impl IntoIterator for &mut Vec<T>  -> Item = &mut T

fn type_signatures() {
    let vec: Vec<String> = vec!["a".into(), "b".into()];

    // Type of iterator and items:
    let iter: std::vec::IntoIter<String> = vec.into_iter();
    // iter.next() returns Option<String>
    for s in iter {
        println!("Owned: {}", s);
    }

    let vec: Vec<String> = vec!["a".into(), "b".into()];
    let iter: std::slice::Iter<'_, String> = vec.iter();
    // iter.next() returns Option<&String>
    for s in iter {
        println!("Borrowed: {}", s);
    }

    let mut vec: Vec<String> = vec!["a".into(), "b".into()];
    let iter: std::slice::IterMut<'_, String> = vec.iter_mut();
    // iter.next() returns Option<&mut String>
    for s in iter {
        s.push_str("!");
    }
    println!("After mut iteration: {:?}", vec);
}

fn main() {
    type_signatures();
    println!("Iterator traits example completed");
}
