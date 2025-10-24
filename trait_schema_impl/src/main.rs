use trait_schema_impl::trait_schema;

#[trait_schema]
trait MyTrait {
    fn my_method(&self) -> String;
}

fn main() {
    println!("{:?}", MyTrait_schema());
}
