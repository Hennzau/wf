use elvwf::Wired;

#[derive(Wired)]
#[wf(struct(header(dsl = "S:8|_:8")))]
struct Bad {
    #[wf(scalar(format(be)), opt(trigger = S))]
    maybe: Option<u32>,
}

fn main() {}
