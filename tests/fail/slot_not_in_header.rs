use elvwf::Wired;

#[derive(Wired)]
#[wf(struct(header(dsl = "A:8|_:8")))]
struct Bad<'a> {
    #[wf(slice(len(slot = S)))]
    content: &'a str,
}

fn main() {}
