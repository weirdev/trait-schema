use trait_schema::trait_schema;

#[trait_schema]
trait MyTrait {
    fn my_method(&self, #[arg(collection_as_item, assert_len = 1)] arg1: Vec<String>) -> String;
}

fn main() {
    println!("{:?}", MyTrait_schema());
}
