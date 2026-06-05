use elvwf::Wired;

#[derive(Wired)]
struct Bad<'a> {
    #[wf(len(slot = S))]
    content: &'a str,
}

fn main() {}
