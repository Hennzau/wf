use elvwf::Wired;

#[derive(Wired)]
#[wf(header(dsl = "A:12"))]
struct Bad {
    #[wf(format = Be)]
    a: u32,
}

fn main() {}
