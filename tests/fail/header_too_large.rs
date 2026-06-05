use elvwf::Wired;

#[derive(Wired)]
#[wf(header(dsl = "A:128"))]
struct Bad {
    #[wf(format = Be)]
    a: u64,
}

fn main() {}
