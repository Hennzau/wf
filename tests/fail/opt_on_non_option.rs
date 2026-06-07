use elvwf::Wired;

#[derive(Wired)]
#[wf(struct(header(dsl = "F:1|_:7")))]
struct Bad {
    #[wf(opt(trigger = F), scalar(format(be)))]
    not_an_option: u32,
}

fn main() {}
