use elvwf::Wired;

#[derive(Wired)]
#[wf(header(dsl = "F:1|_:7"))]
struct Bad {
    #[wf(opt(), format = Be)]
    maybe: Option<u32>,
}

fn main() {}
