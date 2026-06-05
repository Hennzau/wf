use elvwf::Wired;

#[derive(Wired, Clone, Debug, PartialEq)]
struct Inner {
    #[wf(format = Be)]
    x: u32,
}

#[derive(Wired)]
struct Bad {
    #[wf(format = Le)]
    inner: Inner,
}

fn main() {}
