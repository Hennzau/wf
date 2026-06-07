use elvwf::Wired;

#[derive(Wired)]
#[wf(struct(header(dsl = "F:2|_:6")))]
struct Bad {
    #[wf(scalar(format(le)), opt(trigger = F))]
    maybe: Option<u32>,
}

fn main() {}
