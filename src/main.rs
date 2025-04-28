use sawfish::slaspec::builder::SLASpecBuilder;

fn main() {
    let slab = SLASpecBuilder::new();
    println!("{}", slab.build());
}
