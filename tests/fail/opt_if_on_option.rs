use elvwf::Wired;

#[derive(Wired)]
#[wf(header(dsl = "F:1|_:7"))]
struct Bad {
    #[wf(opt(if = None), format = Be)]
    value: Option<u32>,
}

fn main() {}
