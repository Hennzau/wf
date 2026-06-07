use elvwf::Wired;

#[derive(Wired)]
#[wf(struct(header(dsl = "A:12")))]
struct Bad {
    #[wf(scalar(format(be)))]
    a: u32,
}

fn main() {}
