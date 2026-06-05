use elvwf::Wired;

#[derive(Wired)]
#[wf(header(dsl = "F:2|_:6"))]
struct Bad {
    #[wf(opt(flag = F), format = Be)]
    maybe: Option<u32>,
}

fn main() {}
