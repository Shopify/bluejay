use std::error::Error;
use std::io::Write;

#[bluejay_typegen::typegen("examples/schema.graphql", codec = "miniserde")]
#[allow(dead_code)]
pub mod schema {
    type Date = String;
    type DateTime = String;
    type DateTimeWithoutTimezone = String;
    type Decimal = String;
    type Id = String;
    type TimeWithoutTimezone = String;
    type Void = ();

    #[query("examples/input.graphql")]
    pub mod input {}
}

pub fn function(input: schema::input::Input) -> schema::FunctionRunResult {
    schema::FunctionRunResult {
        discount_application_strategy: schema::DiscountApplicationStrategy::Maximum,
        discounts: vec![schema::Discount {
            message: Some("Discount 1".to_string()),
            targets: input
                .cart
                .lines
                .into_iter()
                .filter_map(|cart_line| {
                    (cart_line.quantity > 1).then_some(schema::Target::ProductVariant(
                        schema::ProductVariantTarget {
                            id: cart_line.id,
                            quantity: Some(cart_line.quantity),
                        },
                    ))
                })
                .collect(),
            value: schema::Value::Percentage(schema::Percentage {
                value: "50".to_string(),
            }),
        }],
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut string = String::new();
    std::io::Read::read_to_string(&mut std::io::stdin(), &mut string)?;
    let input: schema::input::Input = bluejay_typegen::miniserde::json::from_str(&string)?;
    let mut out = std::io::stdout();
    let result = function(input);
    let serialized = bluejay_typegen::miniserde::json::to_string(&result);
    out.write_all(serialized.as_bytes())?;
    out.flush()?;

    Ok(())
}
