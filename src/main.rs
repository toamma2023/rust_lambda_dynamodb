use aws_lambda_events::event::s3::S3Event;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{config::Region,types::AttributeValue,Client, Error as DynamoError};

pub struct Parts {
    pub patrs_id: String,
    pub parts_name: String,
    pub manufacture_name: String,
    pub manufacture_number: String,
    pub model_number: String,
    pub spec: String,
    pub stock_quantity: usize,
    pub minimum_stock_quantity: usize,
    pub last_stock_in_date: String,
    pub discontinued_date: String,
    pub last_purchase_date: String,
    pub supplier_name: String,
    pub notes: String,
    pub last_update_date: String
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(_event: LambdaEvent<S3Event>) -> Result<(), Error> {
    // Extract some useful information from the request
    // DynamoDBクライアント用意
    println!("S3 Event Call");
    let region_provider = RegionProviderChain::first_try(Region::new("ap-northeast-1"));
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);
    let parts = Parts {
        patrs_id:"tttt".to_string(),
        parts_name:"tttt".to_string(),
        manufacture_name:"tttt".to_string(),
        manufacture_number:"tttt".to_string(),
        model_number:"tttt".to_string(),
        spec:"tttt".to_string(),
        stock_quantity:2,
        minimum_stock_quantity:5,
        last_stock_in_date:"tttt".to_string(),
        discontinued_date:"tttt".to_string(),
        last_purchase_date:"tttt".to_string(),
        supplier_name:"tttt".to_string(),
        notes:"tttt".to_string(),
        last_update_date:"tttt".to_string(),
    };
    println!("Put call");
    put_item_manually(&client, parts).await?;
    Ok(())
}

async fn put_item_manually(client: &Client, parts: Parts) -> Result<(), DynamoError> {
    let _res = client
        .put_item()
        .table_name("Parts")
        .item("PartsID", AttributeValue::S(parts.patrs_id.to_string()))
        .item("PartsName", AttributeValue::S(parts.parts_name.to_string()))
        .item("ManufactureName", AttributeValue::S(parts.manufacture_name.to_string()))
        .item("ManufactureNumber", AttributeValue::S(parts.manufacture_number.to_string()))
        .item("ModelNumber", AttributeValue::S(parts.model_number.to_string()))
        .item("Spec", AttributeValue::S(parts.spec.to_string()))
        .item("StockQuantity", AttributeValue::N(parts.stock_quantity.to_string()))
        .item("MinimumStockQuantity", AttributeValue::N(parts.minimum_stock_quantity.to_string()))
        .item("LastStockInDate", AttributeValue::S(parts.last_stock_in_date.to_string()))
        .item("DiscontinuedDate", AttributeValue::S(parts.discontinued_date.to_string()))
        .item("LastPurchaseDate", AttributeValue::S(parts.last_purchase_date.to_string()))
        .item("SupplierName", AttributeValue::S(parts.supplier_name.to_string()))
        .item("Notes", AttributeValue::S(parts.notes.to_string()))
        .item("LastUpdateDate", AttributeValue::S(parts.last_update_date.to_string()))
        .send()
        .await?;
    println!("Put OK");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
