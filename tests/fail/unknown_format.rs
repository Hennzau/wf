use elvwf::Wired;

#[derive(Wired)]
struct Bad {
    #[wf(format = Foo)]
    a: u32,
}

fn main() {}
