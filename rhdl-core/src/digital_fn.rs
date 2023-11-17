use crate::{ast::KernelFn, Digital, Kind};

pub trait DigitalFn {
    fn kernel_fn() -> Box<KernelFn>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DigitalSignature {
    pub arguments: Vec<Kind>,
    pub ret: Kind,
}

pub trait Describable<Args> {
    fn describe() -> DigitalSignature;
}

macro_rules! describable {
    ($( $($arg:ident)* => $res:ident), +) => (
        $(
            impl <F, $res, $($arg),*> Describable<($res, $($arg),*)> for F
            where
            F: Fn($($arg),*) -> $res,
            $res: Digital,
            $ ($arg: Digital), *
            {
                fn describe() -> DigitalSignature {
                    DigitalSignature {
                        arguments: vec![$($arg::static_kind(),)*],
                        ret: $res::static_kind(),
                    }
                }
            }
        )+
    )
}

describable!(
    => T1,
    T1 => T2,
    T1 T2 => T3,
    T1 T2 T3 => T4,
    T1 T2 T3 T4 => T5,
    T1 T2 T3 T4 T5 => T6
);

pub fn inspect_digital<F, Args>(_f: F) -> DigitalSignature
where
    F: Describable<Args>,
{
    F::describe()
}
