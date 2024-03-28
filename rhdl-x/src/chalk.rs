trait Clock0 {}

struct A;

impl Clock0 for A {}

struct B;

type C = (A, B);
