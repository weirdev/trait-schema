use trait_schema::trait_schema;

#[trait_schema]
trait MyTrait {
    fn my_method(&self) -> String;
}

fn main() {
    println!("{:?}", MyTrait_schema());
}
