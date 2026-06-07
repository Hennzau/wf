use elvwf::Wired;

#[derive(Wired)]
#[wf(struct(header(dsl = "S:8|_:8")))]
struct Bad<'a> {
    #[wf(slice())]
    content: &'a str,
}

fn main() {}
