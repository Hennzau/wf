use elvwf::Wired;

#[derive(Wired)]
struct Bad<'a> {
    #[wf(slice(len(slot = S)))]
    content: &'a str,
}

fn main() {}
