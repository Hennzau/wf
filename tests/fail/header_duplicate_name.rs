use elvwf::Wired;

#[derive(Wired)]
#[wf(struct(header(dsl = "A:4|A:4|_:8")))]
struct Bad {
    #[wf(scalar(format(be)))]
    a: u32,
}

fn main() {}
