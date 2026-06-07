use elvwf::Wired;

#[derive(Wired)]
struct Bad {
    #[wf(scalar(format(Foo)))]
    a: u32,
}

fn main() {}
