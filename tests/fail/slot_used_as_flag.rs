use elvwf::Wired;

#[derive(Wired)]
#[wf(header(dsl = "S:8|_:8"))]
struct Bad {
    #[wf(opt(flag = S), format = Be)]
    maybe: Option<u32>,
}

fn main() {}
