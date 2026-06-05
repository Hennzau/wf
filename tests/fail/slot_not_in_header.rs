use elvwf::Wired;

#[derive(Wired)]
#[wf(header(dsl = "A:8|_:8"))]
struct Bad<'a> {
    #[wf(len(slot = S))]
    content: &'a str,
}

fn main() {}
