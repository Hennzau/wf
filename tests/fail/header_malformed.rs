use elvwf::Wired;

#[derive(Wired)]
#[wf(header(dsl = "A:|B:8"))]
struct Bad {
    #[wf(format = Be)]
    a: u32,
}

fn main() {}
