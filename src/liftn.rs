trait LiftExt: Sized {
    type Head;

    fn liftn<T, F, A, R>(self, rest: T, f: F) -> R where 
        T: VarLift<<Self as LiftExt>::Head, Arg=A>,
        F: FnOnce(A) -> R;
}

trait VarLift<Head> {
    type Arg;

    fn to_arg(self, head: Head) -> <Self as VarLift<Head>>::Arg;
}

impl<Head, T0> VarLift<Head> for (T0,) {
    type Arg = (Head, T0);

    fn to_arg(self, head: Head) -> (Head, T0) {
        (head, self.0)
    }
}

impl<Head, T0, T1> VarLift<Head> for (T0, T1,) {
    type Arg = (Head, T0, T1);

    fn to_arg(self, head: Head) -> (Head, T0, T1) {
        (head, self.0, self.1)
    }
}

// Just for this test:
//
impl<H> LiftExt for (H,) {
    type Head = H;

    fn liftn<T, F, A, R>(self, rest: T, f: F) -> R where
        T: VarLift<H, Arg=A>,
        F: FnOnce(A) -> R,
    {
        f(rest.to_arg(self.0))
    }
}

fn main() {
    println!("{}", (0,).liftn((1, 2), |(a, b, c)| a+b+c));
}
