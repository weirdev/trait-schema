use std::sync::Arc;

use trait_schema::trait_schema;

#[trait_schema]
trait MyTrait {
    fn my_method(&self, #[arg(collection_as_item, assert_len = 1)] arg1: Vec<String>) -> String;
}

#[trait_schema]
trait SimpleTrait {
    fn method_one(&self, arg1: String) -> i32;
    #[func(cffi_impl_no_op)]
    fn method_two(&self, arg2: i32) -> String;
}

#[trait_schema]
trait AnnotatedTrait {
    fn with_collection(&self, #[arg(collection_as_item)] items: Vec<String>) -> usize;
    fn with_assert_len(&self, #[arg(assert_len = 5)] data: Vec<i32>) -> bool;
    fn with_cffi_type(&self, #[arg(cffi_type = "ptr<f64>")] values: Arc<f64>) -> bool;
}

#[trait_schema]
trait ComplexTrait {
    fn no_args(&self) -> ();
    fn single_arg(&self, value: String) -> String;
    fn multiple_args(&self, x: i32, y: String, z: Vec<String>) -> bool;
    fn annotated_args(
        &self,
        #[arg(collection_as_item, assert_len = 10)] items: Vec<String>,
        #[arg(assert_len = 2)] pair: Vec<i32>,
    ) -> String;
}

#[trait_schema(T = "ptr<void>")]
trait SpecializedTrait<T> {
    fn specialized_method(&self, value: T) -> String;
}

#[trait_schema(T = "ptr<void>")]
trait SpecializedSubTrait<T>: SimpleTrait {
    fn specialized_method(&self, value: T) -> String;
}

fn main() {
    println!("=== MyTrait Schema ===");
    let my_trait_schema = MyTrait_schema();
    println!("{:#?}", my_trait_schema);
    println!("Trait name: {}", my_trait_schema.name);
    for func in &my_trait_schema.functions {
        println!("Function: {}", func);
    }

    println!("\n=== SimpleTrait Schema ===");
    let simple_trait_schema = SimpleTrait_schema();
    println!("{:#?}", simple_trait_schema);
    for func in &simple_trait_schema.functions {
        println!("Function: {}", func);
    }

    println!("\n=== AnnotatedTrait Schema ===");
    let annotated_trait_schema = AnnotatedTrait_schema();
    println!("{:#?}", annotated_trait_schema);
    for func in &annotated_trait_schema.functions {
        println!("Function: {}", func);
        for arg in &func.args {
            if let Some(ann) = &arg.annotations {
                println!(
                    "  Arg: {} - collection_as_item: {}, assert_len: {:?}, cffi_type: {:?}",
                    arg.name, ann.collection_as_item, ann.assert_len, ann.cffi_type
                );
            }
        }
    }

    println!("\n=== ComplexTrait Schema ===");
    let complex_trait_schema = ComplexTrait_schema();
    println!("{:#?}", complex_trait_schema);
    println!("Total functions: {}", complex_trait_schema.functions.len());
    for func in &complex_trait_schema.functions {
        println!("Function: {} with {} args", func.name, func.args.len());
    }

    println!("\n=== SpecializedTrait Schema ===");
    let specialized_trait_schema = SpecializedTrait_schema();
    println!("{:#?}", specialized_trait_schema);
    println!(
        "Generic parameters: {:?}",
        specialized_trait_schema.generics.len()
    );
    for generic in &specialized_trait_schema.generics {
        println!(
            "  Generic: {} - cffi_type: {:?}",
            generic.name,
            generic
                .annotations
                .as_ref()
                .and_then(|a| a.cffi_type.clone())
        );
    }

    println!("\n=== SpecializedSubTrait Schema ===");
    let specialized_trait_schema = SpecializedSubTrait_schema();
    println!("{:#?}", specialized_trait_schema);
}
