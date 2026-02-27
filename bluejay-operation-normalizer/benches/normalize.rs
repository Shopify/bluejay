use bluejay_parser::ast::{executable::ExecutableDocument, Parse};
use criterion::{criterion_group, criterion_main, Criterion};

fn parse(input: &str) -> ExecutableDocument {
    ExecutableDocument::parse(input)
        .result
        .expect("parse error")
}

fn bench_small(c: &mut Criterion) {
    let doc = parse("query { user { name email } }");
    c.bench_function("normalize_small", |b| {
        b.iter(|| bluejay_operation_normalizer::normalize(&doc, None).unwrap())
    });
    c.bench_function("signature_small", |b| {
        b.iter(|| bluejay_operation_normalizer::signature(&doc, None).unwrap())
    });
}

fn bench_medium(c: &mut Criterion) {
    let doc = parse(
        r#"
        query GetUser($id: ID!, $first: Int = 10, $after: String) {
            user(id: $id) {
                name
                email
                avatar
                role
                posts(first: $first, after: $after, orderBy: "created_at") {
                    edges {
                        cursor
                        node {
                            title
                            body
                            createdAt
                            tags
                        }
                    }
                    pageInfo {
                        hasNextPage
                        endCursor
                    }
                }
            }
        }
        "#,
    );
    c.bench_function("normalize_medium", |b| {
        b.iter(|| bluejay_operation_normalizer::normalize(&doc, Some("GetUser")).unwrap())
    });
    c.bench_function("signature_medium", |b| {
        b.iter(|| bluejay_operation_normalizer::signature(&doc, Some("GetUser")).unwrap())
    });
}

fn bench_complex(c: &mut Criterion) {
    let doc = parse(
        r#"
        query ComplexQuery($userId: ID!, $includeEmail: Boolean = true, $limit: Int = 20, $offset: Int = 0) @cacheControl(maxAge: 300) {
            user(id: $userId) {
                ...UserBasic
                ...UserPosts
                followers(limit: $limit, offset: $offset) {
                    ...UserBasic
                    mutualFriends {
                        ...UserBasic
                    }
                }
            }
            systemStatus {
                healthy
                version
                uptime
            }
        }

        fragment UserBasic on User {
            id
            name
            email @include(if: $includeEmail)
            avatar
            role
            createdAt
        }

        fragment UserPosts on User {
            posts(first: 10) {
                edges {
                    cursor
                    node {
                        ...PostDetails
                    }
                }
                pageInfo {
                    hasNextPage
                    hasPreviousPage
                    startCursor
                    endCursor
                }
                totalCount
            }
        }

        fragment PostDetails on Post {
            id
            title
            body
            createdAt
            updatedAt
            author {
                ...UserBasic
            }
            comments(first: 5) {
                edges {
                    node {
                        id
                        body
                        author {
                            name
                        }
                    }
                }
            }
            tags
            likes
        }
        "#,
    );
    c.bench_function("normalize_complex", |b| {
        b.iter(|| bluejay_operation_normalizer::normalize(&doc, Some("ComplexQuery")).unwrap())
    });
    c.bench_function("signature_complex", |b| {
        b.iter(|| bluejay_operation_normalizer::signature(&doc, Some("ComplexQuery")).unwrap())
    });
}

/// Simulates a Relay/Apollo Client app where each component defines a small
/// fragment and the page query composes them. 10 fragments, transitive deps,
/// plus an unused fragment that should be stripped.
fn bench_fragment_colocation(c: &mut Criterion) {
    let doc = parse(
        r#"
        query ProductPage($handle: String!, $first: Int = 10, $after: String) {
            product(handle: $handle) {
                ...ProductHeader
                ...ProductPricing
                ...ProductMedia
                ...ProductVariants
                ...ProductMetafields
                ...ProductSeo
            }
            shop {
                ...ShopInfo
            }
            cart {
                ...CartSummary
            }
        }

        fragment ProductHeader on Product {
            id
            title
            handle
            description
            vendor
            productType
            tags
            createdAt
            updatedAt
        }

        fragment ProductPricing on Product {
            priceRange {
                ...MoneyRange
            }
            compareAtPriceRange {
                ...MoneyRange
            }
        }

        fragment MoneyRange on PriceRange {
            minVariantPrice { ...MoneyFields }
            maxVariantPrice { ...MoneyFields }
        }

        fragment MoneyFields on Money {
            amount
            currencyCode
        }

        fragment ProductMedia on Product {
            images(first: 10) {
                edges {
                    node {
                        id
                        url
                        altText
                        width
                        height
                    }
                }
            }
        }

        fragment ProductVariants on Product {
            variants(first: $first, after: $after) {
                edges {
                    cursor
                    node {
                        id
                        title
                        sku
                        availableForSale
                        price { ...MoneyFields }
                        compareAtPrice { ...MoneyFields }
                        selectedOptions {
                            name
                            value
                        }
                    }
                }
                pageInfo {
                    hasNextPage
                    endCursor
                }
            }
        }

        fragment ProductMetafields on Product {
            metafield1: metafield(namespace: "custom", key: "care_instructions") { value type }
            metafield2: metafield(namespace: "custom", key: "material") { value type }
            metafield3: metafield(namespace: "custom", key: "sizing_guide") { value type }
        }

        fragment ProductSeo on Product {
            seo {
                title
                description
            }
        }

        fragment ShopInfo on Shop {
            name
            primaryDomain { url }
            shipsToCountries
        }

        fragment CartSummary on Cart {
            id
            totalQuantity
            estimatedCost {
                totalAmount { ...MoneyFields }
                subtotalAmount { ...MoneyFields }
                totalTaxAmount { ...MoneyFields }
            }
        }

        fragment UnusedAnalytics on Product {
            id
            title
            vendor
        }
        "#,
    );
    c.bench_function("normalize_fragment_colocation", |b| {
        b.iter(|| bluejay_operation_normalizer::normalize(&doc, Some("ProductPage")).unwrap())
    });
    c.bench_function("signature_fragment_colocation", |b| {
        b.iter(|| bluejay_operation_normalizer::signature(&doc, Some("ProductPage")).unwrap())
    });
}

/// 30 fields in reverse alphabetical order at root level. Worst case for sort.
fn bench_wide_reverse_sorted(c: &mut Criterion) {
    let doc = parse(
        r#"
        query DashboardQuery {
            zones { id }
            yields { id }
            xrefs { id }
            webhooks { id }
            variants { id }
            users { id }
            transactions { id }
            subscriptions { id }
            returns { id }
            quotas { id }
            products { id }
            payments { id }
            orders { id }
            notifications { id }
            metafields { id }
            locations { id }
            inventoryLevels { id }
            images { id }
            hooks { id }
            giftCards { id }
            fulfillments { id }
            events { id }
            discounts { id }
            customers { id }
            collections { id }
            blogs { id }
            articles { id }
        }
        "#,
    );
    c.bench_function("normalize_wide_reverse", |b| {
        b.iter(|| bluejay_operation_normalizer::normalize(&doc, Some("DashboardQuery")).unwrap())
    });
    c.bench_function("signature_wide_reverse", |b| {
        b.iter(|| bluejay_operation_normalizer::signature(&doc, Some("DashboardQuery")).unwrap())
    });
}

criterion_group!(
    benches,
    bench_small,
    bench_medium,
    bench_complex,
    bench_fragment_colocation,
    bench_wide_reverse_sorted,
);
criterion_main!(benches);
