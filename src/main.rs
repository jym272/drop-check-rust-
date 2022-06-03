#![feature(dropck_eyepatch)] //permantely unstable feature
use std::fmt::Debug;
use std::iter::Empty;
use std::ptr::NonNull; //mut ref covariance in T

pub struct Boks<T> {
    p: NonNull<T>,
    _t: std::marker::PhantomData<T>, //it does drop the type parameter T -> signal to the compiler to look for drop impl
}

impl<T> Boks<T> {
    pub fn ny(t: T) -> Boks<T> {
        Boks {
            //SAFETY: box never creates a null pointer
            p: unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(t))) },
            _t: std::marker::PhantomData,
        }
    }
}
// I promised that the code inside Drop that don't access/use the T, but we can drop the T
unsafe impl<#[may_dangle] T> Drop for Boks<T> {
    fn drop(&mut self) {
        unsafe {
            //construct a box from the inner type of the pointer
            // action: destructor of the T and deallocate the box
            //SAFETY: p was constructed from a Box<T>, and has not been freed
            let _: Box<T> = Box::from_raw(self.p.as_mut());

            //we know that we don't access de T in the drop function
        }
    }
}
impl<T> std::ops::Deref for Boks<T> {
    type Target = T;
    fn deref(&self) -> &T {
        //unsafe because we are dereferencing a pointer
        //SAFETY: is valid since it was contructed from a valid pointer through Box which
        //creates aligned pointers, and hasn't been dropped yet, since self is still alive.
        unsafe { &*self.p.as_ref() }
    }
}
impl<T> std::ops::DerefMut for Boks<T> {
    fn deref_mut(&mut self) -> &mut T {
        //unsafe because we are dereferencing a pointer
        //SAFETY: is valid since it was contructed from a valid pointer through Box which
        //creates aligned pointers, and hasn't been dropped yet, since self is still alive.
        //Since wa hace &mut self, no other mutable reference has been given out to p
        unsafe { &mut *self.p.as_mut() }
    }
}

struct Oisann<T: Debug>(T);

impl<T> Drop for Oisann<T>
where
    T: Debug,
{
    fn drop(&mut self) {
        //access the inner type when is dropped
        println!("{:?}", self.0);
    }
}

struct EmptyIterator<T> {
    _t: std::marker::PhantomData<T>,
}
impl<T> Default for EmptyIterator<T> {
    fn default() -> Self {
        EmptyIterator {
            _t: std::marker::PhantomData,
        }
    }
}
impl<T> Iterator for EmptyIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

fn main() {
    let x = 42;
    let b = Boks::ny(x);
    println!("{:?}", *b);

    //normal box
    let mut y = 42;
    let b = Boks::ny(&mut y);
    println!("{:?}", y);

    //Oisann
    let mut z = 42;
    let b = Boks::ny(Oisann(&mut z));
    // let b = Box::new(Oisann(&mut z));
    // println!("{:?}", z);

    let s = String::from("hei");
    let mut books1 = Boks::ny(&*s);
    let books2: Boks<&'static str> = Boks::ny("heissan");
    books1 = books2;

    //empty iter
    let mut a = 42;
    let mut it: EmptyIterator<Oisann<&'static mut i32>> = EmptyIterator::default();
    let mut o: Option<Oisann<&mut i32>> = Some(Oisann(&mut a));
    {
        o = it.next()
    }
    // &'a mut T is invariant in T, but covariant in 'a
    drop(o);
    println!("{:?}", a);
    let _ = it.next();
}
