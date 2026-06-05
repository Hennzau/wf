use elvwf::Wired;

#[derive(Wired)]
#[wf(header(dsl = "F:1|_:7"))]
struct Bad {
    #[wf(opt(flag = F), format = Be)]
    not_an_option: u32,
}

fn main() {}
