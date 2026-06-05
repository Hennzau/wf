use elvwf::Wired;

#[derive(Wired)]
#[wf(header(dsl = "A:4|A:4|_:8"))]
struct Bad {
    #[wf(format = Be)]
    a: u32,
}

fn main() {}
