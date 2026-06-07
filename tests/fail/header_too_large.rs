use elvwf::Wired;

#[derive(Wired)]
#[wf(struct(header(dsl = "A:128")))]
struct Bad {
    #[wf(scalar(format(be)))]
    a: u64,
}

fn main() {}
