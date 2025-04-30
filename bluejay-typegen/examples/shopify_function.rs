use std::error::Error;
use std::io::Write;

#[derive(bluejay_typegen::serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "bluejay_typegen::serde")]
#[serde(rename_all = "camelCase")]
struct Configuration {
    threshold_quantity: i32,
}

#[bluejay_typegen::typegen("examples/schema.graphql")]
#[allow(dead_code)]
pub mod schema {
    type Date = String;
    type DateTime = String;
    type DateTimeWithoutTimezone = String;
    type Decimal = String;
    type Handle = String;
    type Id = String;
    type Json = serde_json::Value;
    type TimeWithoutTimezone = String;
    type Void = ();

    #[query(
        "examples/input.graphql",
        custom_scalar_overrides = {
            "Input.discountNode.configuration.jsonValue" => super::Configuration,
        }
    )]
    pub mod input {}
}

pub fn function(input: schema::input::Input) -> schema::FunctionRunResult {
    let threshold_quantity = input
        .discount_node
        .configuration
        .map_or(1, |configuration| {
            configuration.json_value.threshold_quantity
        });

    schema::FunctionRunResult {
        discount_application_strategy: schema::DiscountApplicationStrategy::Maximum,
        discounts: vec![schema::Discount {
            message: Some("Discount 1".to_string()),
            targets: input
                .cart
                .lines
                .into_iter()
                .filter_map(|cart_line| {
                    (cart_line.quantity > threshold_quantity).then_some(
                        schema::Target::ProductVariant(schema::ProductVariantTarget {
                            id: cart_line.id,
                            quantity: Some(cart_line.quantity),
                        }),
                    )
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
    let input: schema::input::Input = serde_json::from_str(&string)?;
    let mut out = std::io::stdout();
    let result = function(input);
    let serialized = serde_json::to_string(&result)?;
    out.write_all(serialized.as_bytes())?;
    out.flush()?;

    Ok(())
}
