use std::error::Error;
use std::io::Write;

type Float = f64;

#[bluejay_typegen::typegen("examples/schema.graphql")]
#[allow(dead_code)]
pub mod schema {
    type Date = String;
    type DateTime = String;
    type DateTimeWithoutTimezone = String;
    type Decimal = String;
    type Id = String;
    type TimeWithoutTimezone = String;
    type Void = ();
    type Handle = String;
    type Json = serde_json::Value;

    #[query(
        "examples/input.graphql",
        custom_scalar_overrides = {
            "Input.discountNode.metafield.jsonValue" => super::Float,
        }
    )]
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
                value: input.discount_node.metafield.map_or_else(
                    || "50".to_string(),
                    |metafield| metafield.json_value.to_string(),
                ),
            }),
        }],
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut string = String::new();
    std::io::Read::read_to_string(&mut std::io::stdin(), &mut string)?;
    let input: schema::input::Input = serde_json::from_str(&string)?;
    let mut out = std::io::stdout();
    let result = function(input);
    let serialized = serde_json::to_string(&result)?;
    out.write_all(serialized.as_bytes())?;
    out.flush()?;

    Ok(())
}
