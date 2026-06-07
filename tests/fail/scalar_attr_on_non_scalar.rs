use elvwf::Wired;

#[derive(Wired, Clone, Debug, PartialEq)]
struct Inner {
    #[wf(scalar(format(be)))]
    x: u32,
}

#[derive(Wired)]
struct Bad {
    #[wf(scalar(format(le)))]
    inner: Inner,
}

fn main() {}
