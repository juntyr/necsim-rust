mod private {
    pub trait Sealed {}

    impl Sealed for super::True {}
    impl Sealed for super::False {}
}

pub trait Boolean: private::Sealed {
    const VALUE: bool;
}

pub struct False;
impl Boolean for False {
    const VALUE: bool = false;
}

pub struct True;
impl Boolean for True {
    const VALUE: bool = true;
}

pub trait Or<O: Boolean>: Boolean {
    type RESULT: Boolean;
}

impl<O: Boolean> Or<O> for False {
    type RESULT = O;
}

impl<O: Boolean> Or<O> for True {
    type RESULT = True;
}

pub trait And<O: Boolean>: Boolean {
    type RESULT: Boolean;
}

impl<O: Boolean> And<O> for False {
    type RESULT = False;
}

impl<O: Boolean> And<O> for True {
    type RESULT = O;
}
