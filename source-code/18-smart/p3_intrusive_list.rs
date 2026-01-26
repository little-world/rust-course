// Pattern 3: Intrusive Data Structures - Singly-Linked List
use std::ptr;
use std::marker::PhantomData;

struct ListNode<T> {
    next: *mut ListNode<T>,
    data: T,
}

struct IntrusiveList<T> {
    head: *mut ListNode<T>,
    _phantom: PhantomData<T>,
}

impl<T> IntrusiveList<T> {
    fn new() -> Self {
        IntrusiveList { head: ptr::null_mut(), _phantom: PhantomData }
    }

    fn push_front(&mut self, data: T) {
        let node = Box::into_raw(Box::new(ListNode {
            next: self.head,
            data,
        }));
        self.head = node;
    }

    fn pop_front(&mut self) -> Option<T> {
        if self.head.is_null() {
            return None;
        }
        unsafe {
            let node = Box::from_raw(self.head);
            self.head = node.next;
            Some(node.data)
        }
    }

    fn iter(&self) -> impl Iterator<Item = &T> {
        struct Iter<'a, T> {
            current: *mut ListNode<T>,
            _phantom: PhantomData<&'a T>,
        }
        impl<'a, T> Iterator for Iter<'a, T> {
            type Item = &'a T;
            fn next(&mut self) -> Option<Self::Item> {
                if self.current.is_null() {
                    None
                } else {
                    unsafe {
                        let data = &(*self.current).data;
                        self.current = (*self.current).next;
                        Some(data)
                    }
                }
            }
        }
        Iter { current: self.head, _phantom: PhantomData }
    }
}

impl<T> Drop for IntrusiveList<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

fn main() {
    // Usage
    let mut list = IntrusiveList::new();
    list.push_front(3);
    list.push_front(2);
    list.push_front(1);
    for item in list.iter() {
        println!("{}", item); // 1, 2, 3
    }

    println!("Intrusive list example completed");
}
