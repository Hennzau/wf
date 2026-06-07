use elvwf::Wired;

#[derive(Wired)]
#[wf(struct(header(dsl = "F:1|_:7")))]
struct Bad {
    #[wf(scalar(format(be)), opt(trigger = H))]
    maybe: Option<u32>,
}

fn main() {}
