use aws_lambda_events::event::s3::{S3Event};
use aws_lambda_events::s3::S3EventRecord;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{config::Region,types::AttributeValue,Client as DynamoClient, Error as DynamoError};
use calamine::{Reader, Xlsx, DataType};
use std::io::Cursor;
use aws_sdk_s3 as s3;
use std::fmt; // Import `fmt`

pub struct Parts {
    pub patrs_id                : String,
    pub parts_name              : String,
    pub manufacture_name        : String,
    pub manufacture_number      : String,
    pub model_number            : String,
    pub spec                    : String,
    pub stock_quantity          : usize,
    pub minimum_stock_quantity  : usize,
    pub last_stock_in_date      : String,
    pub discontinued_date       : String,
    pub last_purchase_date      : String,
    pub alternate_parts_id      : String,
    pub supplier_name           : String,
    pub supplier_contact        : String,
    pub notes                   : String,
    pub last_update_date        : String
}

impl Parts {
    fn new(patrs_id: String, parts_name: String, manufacture_name: String, manufacture_number: String, 
        model_number: String, spec: String, stock_quantity: usize, minimum_stock_quantity: usize, 
        last_stock_in_date: String, discontinued_date: String, last_purchase_date: String,
        alternate_parts_id: String, supplier_name: String, supplier_contact: String,
        notes: String,last_update_date: String
    ) -> Self {
        Parts {
            patrs_id              ,
            parts_name            ,
            manufacture_name      ,
            manufacture_number    ,
            model_number          ,
            spec                  ,
            stock_quantity        ,
            minimum_stock_quantity,
            last_stock_in_date    ,
            discontinued_date     ,
            last_purchase_date    ,
            alternate_parts_id    ,
            supplier_name         ,
            supplier_contact      ,
            notes                 ,
            last_update_date      
        }
    }
}

// Similarly, implement `Display` for `Point2D`.
impl fmt::Display for Parts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Customize so only `x` and `y` are denoted.
        write!(f, "parts_name: {}, manufacture_name: {}, manufacture_number: {}, model_number: {},
        spec: {}, stock_quantity: {}, minimum_stock_quantity: {}, last_stock_in_date: {}, 
        discontinued_date: {}, last_purchase_date: {},
        alternate_parts_id: {}, supplier_name: {}, supplier_contact: {}, notes: {}, last_update_date: {}"
        ,self.parts_name, self.manufacture_name, self.manufacture_number, self.model_number 
        ,self.spec, self.stock_quantity, self.minimum_stock_quantity, self.last_stock_in_date, self.discontinued_date 
        ,self.last_purchase_date, self.alternate_parts_id, self.supplier_name, self.supplier_contact 
        ,self.notes, self.last_update_date)
    }
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(s3_event: LambdaEvent<S3Event>) -> Result<(), Error> {
    // Extract some useful information from the request
    println!("S3 Event Call");

    /* S3 Event から対象の record を取得 */
    let record = get_record_from_event(s3_event,String::from("ObjectCreated:Put"))
                          .ok_or("record error!".to_owned())?;
    /* region 取得 */
    let region = Region::new(record.aws_region.ok_or("aws_region error!".to_owned())?);
    /* region provider 取得 */
    let region_provider = RegionProviderChain::first_try(region); /* get region */
    /* config 取得 */
    let config = aws_config::from_env().region(region_provider).load().await; /* set config */
    /* s3 client 取得 */
    let s3_client = s3::Client::new(&config); /* get s3 client */
    /* dynamo client 取得 */
    let dynamo_client = DynamoClient::new(&config);
    /* bucket名 取得 */
    let bucket_name = record.s3.bucket.name.ok_or("bucket.name error!".to_owned())?; /* get bucket */
    println!("bucket name: {}", bucket_name);
    /* key(file)名 取得 */
    let object_key = record.s3.object.key.ok_or("object.key error!".to_owned())?;   /* get key */
    println!("key name: {}", object_key);
    /* 以下 excelオブジェクトを取得する一連の手順 */
    let resp = s3_client.get_object().bucket(bucket_name).key(object_key).send().await?;
    let body = resp.body.collect().await?;
    let bytes: Vec<u8> = body.to_vec();  // Assuming AggregatedBytes can be converted into Vec<u8>
    let cursor = Cursor::new(bytes);
    let mut excel: Xlsx<_> = Xlsx::new(cursor).map_err(|_| "cursor error!".to_owned())?;
    println!("Open Excel");
    // シートの名前を指定
    let sheet_name = "Sheet1";
    // シートを選択
    let range = excel.worksheet_range(sheet_name).ok_or("Failed to open worksheet")?;
    println!("Select Sheet");

    // セルの値を表示
    for row in range?.rows().skip(1) {
        let parts = Parts::new( row_to_string(row.get(0)),
                                row_to_string(row.get(1)),
                                row_to_string(row.get(2)),
                                row_to_string(row.get(3)),
                                row_to_string(row.get(4)),
                                row_to_string(row.get(5)),
                                row_to_usize(row.get(6)),
                                row_to_usize(row.get(7)),
                                row_to_string(row.get(8)),
                                row_to_string(row.get(9)),
                                row_to_string(row.get(10)),
                                row_to_string(row.get(11)),
                                row_to_string(row.get(12)),
                                row_to_string(row.get(13)),
                                row_to_string(row.get(14)),
                                row_to_string(row.get(15)),
                            );
        println!("Display: {}", parts);
        put_item_manually(&dynamo_client, parts).await?;
    }


    println!("Put call");
    Ok(())
}

fn get_record_from_event(s3_event:LambdaEvent<S3Event>,event_type:String)-> Option<S3EventRecord>    {
    for record in s3_event.payload.records {
        let event_name : String =  record.clone().event_name?;
        println!("event name: {}", event_name);
        if event_name == event_type {
            return Some(record);
        }
    }
    None
}

/* 課題　Date型(YYY/MM/DDが変換できない) */
fn row_to_string(data:Option<&DataType>) ->  String{
    // Option<&DataType> から String への変換
    let result: String = match data {
        Some(DataType::String(s)) => s.clone(), // String はクローン可能
        Some(DataType::Int(i)) => i.to_string(), // Int から String への変換
        Some(DataType::Float(f)) => f.to_string(), // Float から String への変換
        Some(DataType::Bool(b)) => b.to_string(), // Bool から String への変換
        Some(DataType::DateTimeIso(t)) => t.clone(), // Bool から String への変換
        //Some(DateTime(d)) => data.unwrap().clone().as_datetime().map(|dt| dt.to_string()),
        _ => "CantConvert".to_string(), // 上記以外の DataType は変換できないので "" を返す
    };
    return result;
}

fn row_to_usize(data:Option<&DataType>) ->  usize{
    // Option<&DataType> から useize  への変換
    let result: usize = match data {
        Some(DataType::Int(i)) => {
            // DataType::Int を usize に変換
            *i as usize
        },
        _ => 0, // 整数でない場合、変換不可能なので 0 を返す
    };
    return result;
}


async fn put_item_manually(client: &DynamoClient, parts: Parts) -> Result<(), DynamoError> {
    println!("put item");
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
        .item("AlternatePartsID", AttributeValue::S(parts.last_purchase_date.to_string()))
        .item("SupplierName", AttributeValue::S(parts.supplier_name.to_string()))
        .item("SupplierContact", AttributeValue::S(parts.supplier_name.to_string()))
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
