use elvwf::Wired;

#[derive(Wired)]
#[wf(header(dsl = "S:8|_:8"))]
struct Bad<'a> {
    #[wf(len(slot = S, prefixed(u16)))]
    content: &'a str,
}

fn main() {}
